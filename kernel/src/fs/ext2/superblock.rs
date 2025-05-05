use alloc::vec::Vec;

use crate::{allocator::bump, serial_println};

use super::superblock_data_types as sbdt;

#[derive(Debug)]
#[repr(C)]
struct SuperblockExt {
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
impl SuperblockExt {
    unsafe fn to_bytes(&self) -> &[u8] {
        core::slice::from_raw_parts(
            (self as *const SuperblockExt) as *const u8,
            ::core::mem::size_of::<SuperblockExt>(),
        )
    }
}

#[derive(Debug)]
#[repr(C)]
struct SuperblockBase {
    inodes_count: u32,            // total number of inodes
    blocks_count: u32,            // total number of blocks
    reserved_blocks_count: u32,   // reserved for superuser
    free_blocks_count: u32,       // unallocated blocks
    free_inodes_count: u32,       // unallocated inodes
    superblock_block_number: u32, // block number of the superblock
    block_size: u32,              // size of a block in bytes
    fragment_size: u32,           // size of a fragment in bytes
    blocks_per_group: u32,        // number of blocks in a block group
    fragments_per_group: u32,     // number of fragments in a block group
    inodes_per_group: u32,        // number of inodes in a block group
    last_mount_time: u32,         // time of last mount
    last_write_time: u32,         // time of last write
    mount_count: u16,             // number of mounts since last check
    max_mount_count: u16,         // max mounts before check
    signature: u16,               // signature of the filesystem (should be 0xEF53 for ext2)
    fs_state: u16,                // state of the filesystem (0 = clean, 1 = errors)
    error_handling: u16,          // error handling (0 = ignore, 1 = remount read-only, 2 = panic)
    minor_version: u16,           // minor version of the filesystem
    last_check_time: u32,         // time of last check
    check_interval: u32,          // max time between checks
    creator_os: u32,              // OS that created the filesystem
    major_version: u32,           // major version of the filesystem
    uid_reserved: u16,            // uid that can use reserved blocks
    gid_reserved: u16,            // gid that can use reserved blocks}
}
impl SuperblockBase {
    unsafe fn to_bytes(&self) -> &[u8] {
        core::slice::from_raw_parts(
            (self as *const SuperblockBase) as *const u8,
            ::core::mem::size_of::<SuperblockBase>(),
        )
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct Superblock {
    base: SuperblockBase,           // base fields
    extended_fields: SuperblockExt, // extended fields
    block_groups_count: u32,        // number of block groups
}
impl Superblock {
    pub fn new() -> Self {
        use crate::drivers::ata;
        let mut buf = [0u8; 512];
        let mut read_result = ata::read(0, 2, &mut buf);
        if read_result.is_err() {
            panic!("Failed to read from disk");
        }
        let base_ptr = buf[0..84].as_ptr();
        let ext_ptr = buf[84..236].as_ptr();
        let base = unsafe { core::ptr::read(base_ptr as *const SuperblockBase) };
        let ext = unsafe { core::ptr::read(ext_ptr as *const SuperblockExt) };
        let met_1 = base.blocks_count / base.blocks_per_group;
        let met_2 = base.inodes_count / base.inodes_per_group;
        if met_1 != met_2 {
            panic!("Inconsistent number of block groups");
        }
        let block_groups_count = met_1;
        Superblock {
            base,
            extended_fields: ext,
            block_groups_count,
        }
    }

    pub fn get_block_size(&self) -> usize {
        return 1024 << self.base.block_size;
    }

    pub fn get_inode_size(&self) -> usize {
        return self.extended_fields.inode_size as usize;
    }

    pub fn get_inodes_per_group(&self) -> usize {
        return self.base.inodes_per_group as usize;
    }

    pub fn get_block_group_count(&self) -> usize {
        return self.block_groups_count as usize;
    }

    pub fn get_blocks_per_group(&self) -> usize {
        return self.base.blocks_per_group as usize;
    }

    unsafe fn to_bytes(&self) -> Vec<u8> {
        let base = self.base.to_bytes();
        let ext = self.extended_fields.to_bytes();
        let base_size = core::mem::size_of::<SuperblockBase>();
        let ext_size = core::mem::size_of::<SuperblockExt>();
        let total_size = base_size + ext_size;
        let mut data = Vec::with_capacity(total_size);
        data.extend_from_slice(base);
        data.extend_from_slice(ext);
        data
    }

    pub fn flush(&self) {
        //TODO Remove?
        unsafe {
            use crate::drivers::ata;
            let self_data = self.to_bytes();
            let mut disk_data = Vec::with_capacity(1024);
            let mut buf = [0u8; 512];
            for i in 2..4 {
                let read_result = ata::read(0, i, &mut buf);
                if read_result.is_err() {
                    panic!("Failed to read from disk");
                }
                for j in 0..512 {
                    disk_data.push(buf[j]);
                }
            }
            let mut write_flag = false;
            for i in 0..self_data.len() {
                if self_data[i] != disk_data[i] {
                    serial_println!(
                        "Data mismatch at index {}: {} != {}",
                        i,
                        self_data[i],
                        disk_data[i]
                    );
                    disk_data[i] = self_data[i];
                    write_flag = true;
                }
            }
            if write_flag {
                let write_buf = &disk_data[0..512];
                let write_result = ata::write(0, 2, &write_buf);
                if write_result.is_err() {
                    panic!("Failed to write to disk");
                }
                let write_buf = &disk_data[512..1024];
                let write_result = ata::write(0, 3, &write_buf);
                if write_result.is_err() {
                    panic!("Failed to write to disk");
                }
                serial_println!("Superblock flushed to disk");
            } else {
                serial_println!("No changes to superblock, not flushing to disk");
            }
        }
    }

    pub fn get_free_blocks(&self) -> u32 {
        self.base.free_blocks_count
    }

    pub fn set_free_blocks(&mut self, new_value: u32) {
        self.base.free_blocks_count = new_value;
    }

    pub fn get_first_free_data_block(&self) -> u32 {
        return self.base.superblock_block_number;
    }
}
