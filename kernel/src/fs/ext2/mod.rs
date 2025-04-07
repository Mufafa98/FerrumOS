use crate::drivers::*;
use crate::io::serial;
use crate::{serial_print, serial_println};

use alloc::vec::Vec;

mod superblock_data_types;
use superblock_data_types as sbdt;

#[derive(Debug)]
#[repr(C)]
struct SuperblockExtendedFields {
    first_n_r_inode: u32,             // first non-reserved inode
    inode_size: u16,                  // size of an inode structure in bytes
    block_group_number: u16,          // block group number of the superblock
    optional_features: sbdt::Feature, // optional features supported
    required_features: sbdt::Feature, // required features supported
    readonly_features: sbdt::Feature, // features that if not present, the drive should be read-only
    fs_id: [u8; 16],                  // filesystem ID
    volume_name: [u8; 16],            // volume name
    last_mounted: [u8; 64],           // last mounted path
    compression: u32,                 // compression algorithm used
    prealloc_blocks_f: u8,            // number of blocks to preallocate for
    prealloc_blocks_d: u8,            // number of directories to preallocate for
    unused: u16,                      // unused
    journal_id: [u8; 16],             // journal ID
    journal_inum: u32,                // journal inode number
    journal_dev: u32,                 // journal device number
    head_orphan: u32,                 // head of orphan list
}
impl SuperblockExtendedFields {
    fn try_from_bytes(buf: &[u8]) -> Self {
        let mut temp: SuperblockExtendedFields =
            unsafe { core::ptr::read(buf.as_ptr() as *const _) };
        temp
    }
}

#[derive(Debug)]
#[repr(C)]
struct Superblock {
    inodes_count: u32,                                 // total number of inodes
    blocks_count: u32,                                 // total number of blocks
    reserved_blocks_count: u32,                        // reserved for superuser
    free_blocks_count: u32,                            // unallocated blocks
    free_inodes_count: u32,                            // unallocated inodes
    superblock_block_number: u32,                      // block number of the superblock
    block_size: u32,                                   // size of a block in bytes
    fragment_size: u32,                                // size of a fragment in bytes
    blocks_per_group: u32,                             // number of blocks in a block group
    fragments_per_group: u32,                          // number of fragments in a block group
    inodes_per_group: u32,                             // number of inodes in a block group
    last_mount_time: u32,                              // time of last mount
    last_write_time: u32,                              // time of last write
    mount_count: u16,                                  // number of mounts since last check
    max_mount_count: u16,                              // max mounts before check
    signature: u16,       // signature of the filesystem (should be 0xEF53 for ext2)
    fs_state: u16,        // state of the filesystem (0 = clean, 1 = errors)
    error_handling: u16,  // error handling (0 = ignore, 1 = remount read-only, 2 = panic)
    minor_version: u16,   // minor version of the filesystem
    last_check_time: u32, // time of last check
    check_interval: u32,  // max time between checks
    creator_os: u32,      // OS that created the filesystem
    major_version: u32,   // major version of the filesystem
    uid_reserved: u16,    // uid that can use reserved blocks
    gid_reserved: u16,    // gid that can use reserved blocks
    extended_fields: Option<SuperblockExtendedFields>, // extended fields
    block_groups_count: u32, // number of block groups
}
impl Superblock {
    fn try_from_bytes(buf: &[u8]) -> Self {
        let mut sb: Superblock = unsafe { core::ptr::read(buf.as_ptr() as *const _) };
        let mut temp_ext: SuperblockExtendedFields =
            SuperblockExtendedFields::try_from_bytes(&buf[84..236]);
        sb.extended_fields = Some(temp_ext);
        let met_1 = sb.blocks_count / sb.blocks_per_group;
        let met_2 = sb.inodes_count / sb.inodes_per_group;
        if met_1 != met_2 {
            panic!("Inconsistent number of block groups");
        }
        sb.block_groups_count = met_1;
        sb
    }
}

#[derive(Debug)]
#[repr(C)]
struct BlockGroupDescriptor {
    block_usage_bitmap: u32, // block address of block usage bitmap
    inode_usage_bitmap: u32, // block address of inode usage bitmap
    inode_table: u32,        // block address of inode table
    free_blocks_count: u16,  // free blocks count
    free_inodes_count: u16,  // free inodes count
    directories_count: u16,  // directories count
}
impl BlockGroupDescriptor {
    fn try_from_bytes(buf: &[u8]) -> Self {
        let bgd: BlockGroupDescriptor = unsafe { core::ptr::read(buf.as_ptr() as *const _) };
        bgd
    }
}

#[derive(Debug)]
#[repr(C)]
struct BlockGroupDescriptorTable {
    block_group_descriptors: Vec<BlockGroupDescriptor>,
}
impl BlockGroupDescriptorTable {
    fn try_from_bytes(buf: &[u8], block_count: usize) -> Self {
        let mut bgdt: BlockGroupDescriptorTable = BlockGroupDescriptorTable {
            block_group_descriptors: Vec::new(),
        };
        for block in 0..block_count {
            let start = block * 32;
            let bgd: BlockGroupDescriptor =
                BlockGroupDescriptor::try_from_bytes(&buf[start as usize..start as usize + 32]);
            bgdt.block_group_descriptors.push(bgd);
        }
        bgdt
    }
}

pub fn init() {
    let mut buf = [0u8; 512];
    let mut read_result = ata::read(0, 2, &mut buf);
    // LBA 3 does not contain any relevant information
    if read_result.is_err() {
        panic!("Failed to read from disk");
    }

    let sb = Superblock::try_from_bytes(&buf);
    // serial_println!("{:?}", sb);

    let mut data: Vec<u8> = Vec::new();
    for lba in 4..=5 {
        read_result = ata::read(0, lba, &mut buf);
        if read_result.is_err() {
            panic!("Failed to read from disk");
        }
        data.extend_from_slice(&buf);
    }

    let bgdt = BlockGroupDescriptorTable::try_from_bytes(&data, sb.block_groups_count as usize);
}
