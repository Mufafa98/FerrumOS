use crate::drivers::*;
use crate::io::serial;
use crate::{serial_print, serial_println};

use alloc::vec::Vec;

mod superblock_data_types;
use superblock_data_types as sbdt;

#[derive(Debug)]
struct SuperblockExtendedFields {
    first_n_r_inode: u32,              // first non-reserved inode
    inode_size: u16,                   // size of an inode structure in bytes
    block_group_number: u16,           // block group number of the superblock
    optional_features: sbdt::Feature,  // optional features supported
    required_features: sbdt::Feature,  // required features supported
    readonly_features: sbdt::Feature, // features that if not present, the drive should be read-only
    fs_id: [u8; 16],                  // filesystem ID
    volume_name: [u8; 16],            // volume name
    last_mounted: [u8; 64],           // last mounted path
    compression: sbdt::CompressionAlg, // compression algorithm used
    prealloc_blocks_f: u8,            // number of blocks to preallocate for
    prealloc_blocks_d: u8,            // number of directories to preallocate for
    journal_id: [u8; 16],             // journal ID
    journal_inum: u32,                // journal inode number
    journal_dev: u32,                 // journal device number
    head_orphan: u32,                 // head of orphan list
}
impl SuperblockExtendedFields {
    fn new(buf: &[u8]) -> Self {
        SuperblockExtendedFields {
            first_n_r_inode: u32::from_le_bytes(buf[84..88].try_into().unwrap()),
            inode_size: u16::from_le_bytes(buf[88..90].try_into().unwrap()),
            block_group_number: u16::from_le_bytes(buf[90..92].try_into().unwrap()),
            optional_features: sbdt::Feature::new(u32::from_le_bytes(
                buf[92..96].try_into().unwrap(),
            )),
            required_features: sbdt::Feature::new(u32::from_le_bytes(
                buf[96..100].try_into().unwrap(),
            )),
            readonly_features: sbdt::Feature::new(u32::from_le_bytes(
                buf[100..104].try_into().unwrap(),
            )),
            fs_id: buf[104..120].try_into().unwrap(),
            volume_name: buf[120..136].try_into().unwrap(),
            last_mounted: buf[136..200].try_into().unwrap(),
            compression: sbdt::CompressionAlg::from_u32(u32::from_le_bytes(
                buf[200..204].try_into().unwrap(),
            )),
            prealloc_blocks_f: buf[204],
            prealloc_blocks_d: buf[205],
            journal_id: buf[208..224].try_into().unwrap(),
            journal_inum: u32::from_le_bytes(buf[224..228].try_into().unwrap()),
            journal_dev: u32::from_le_bytes(buf[228..232].try_into().unwrap()),
            head_orphan: u32::from_le_bytes(buf[232..236].try_into().unwrap()),
        }
    }
}

#[derive(Debug)]
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
    signature: u16,        // signature of the filesystem (should be 0xEF53 for ext2)
    fs_state: sbdt::State, // state of the filesystem (0 = clean, 1 = errors)
    error_handling: sbdt::Error, // error handling (0 = ignore, 1 = remount read-only, 2 = panic)
    minor_version: u16,    // minor version of the filesystem
    last_check_time: u32,  // time of last check
    check_interval: u32,   // max time between checks
    creator_os: sbdt::CreatorOS, // OS that created the filesystem
    major_version: u32,    // major version of the filesystem
    uid_reserved: u16,     // uid that can use reserved blocks
    gid_reserved: u16,     // gid that can use reserved blocks
    extended_fields: Option<SuperblockExtendedFields>, // extended fields
    block_groups_count: u32, // number of block groups
}
impl Superblock {
    fn new(buf: &[u8]) -> Self {
        let mut sb = Superblock {
            inodes_count: u32::from_le_bytes(buf[0..4].try_into().unwrap()),
            blocks_count: u32::from_le_bytes(buf[4..8].try_into().unwrap()),
            reserved_blocks_count: u32::from_le_bytes(buf[8..12].try_into().unwrap()),
            free_blocks_count: u32::from_le_bytes(buf[12..16].try_into().unwrap()),
            free_inodes_count: u32::from_le_bytes(buf[16..20].try_into().unwrap()),
            superblock_block_number: u32::from_le_bytes(buf[20..24].try_into().unwrap()),
            block_size: u32::from_le_bytes(buf[24..28].try_into().unwrap()),
            fragment_size: u32::from_le_bytes(buf[28..32].try_into().unwrap()),
            blocks_per_group: u32::from_le_bytes(buf[32..36].try_into().unwrap()),
            fragments_per_group: u32::from_le_bytes(buf[36..40].try_into().unwrap()),
            inodes_per_group: u32::from_le_bytes(buf[40..44].try_into().unwrap()),
            last_mount_time: u32::from_le_bytes(buf[44..48].try_into().unwrap()),
            last_write_time: u32::from_le_bytes(buf[48..52].try_into().unwrap()),
            mount_count: u16::from_le_bytes(buf[52..54].try_into().unwrap()),
            max_mount_count: u16::from_le_bytes(buf[54..56].try_into().unwrap()),
            signature: u16::from_le_bytes(buf[56..58].try_into().unwrap()),
            fs_state: sbdt::State::from_u16(u16::from_le_bytes(buf[58..60].try_into().unwrap())),
            error_handling: sbdt::Error::from_u16(u16::from_le_bytes(
                buf[60..62].try_into().unwrap(),
            )),
            minor_version: u16::from_le_bytes(buf[62..64].try_into().unwrap()),
            last_check_time: u32::from_le_bytes(buf[64..68].try_into().unwrap()),
            check_interval: u32::from_le_bytes(buf[68..72].try_into().unwrap()),
            creator_os: sbdt::CreatorOS::from_u32(u32::from_le_bytes(
                buf[72..76].try_into().unwrap(),
            )),
            major_version: u32::from_le_bytes(buf[76..80].try_into().unwrap()),
            uid_reserved: u16::from_le_bytes(buf[80..82].try_into().unwrap()),
            gid_reserved: u16::from_le_bytes(buf[82..84].try_into().unwrap()),
            extended_fields: None,
            block_groups_count: 0,
        };
        if sb.major_version >= 1 {
            sb.extended_fields = Some(SuperblockExtendedFields::new(&buf));
        }
        let met_1 = sb.blocks_count / sb.blocks_per_group;
        let met_2 = sb.inodes_count / sb.inodes_per_group;
        if met_1 != met_2 {
            panic!("Inconsistent number of block groups");
        }
        sb.block_groups_count = met_1;
        sb
    }
}

pub fn init() {
    let mut buf = [0u8; 512];
    let read_result = ata::read(0, 2, &mut buf);
    // LBA 3 does not contain any relevant information
    if read_result.is_err() {
        panic!("Failed to read from disk");
    }

    let sb = Superblock::new(&buf);
    serial_println!("{:?}", sb);
    // serial_println!("{:?}", sb);

    // fn temp() {
    //     let read_result = ata::read(0, 4, &mut buf);
    //     if read_result.is_err() {
    //         panic!("Failed to read from disk");
    //     }
    //     let mut block: Vec<u8> = Vec::new();
    //     for el in buf.iter() {
    //         block.push(*el);
    //     }
    //     let read_result = ata::read(0, 5, &mut buf);
    //     if read_result.is_err() {
    //         panic!("Failed to read from disk");
    //     }
    //     for el in buf.iter() {
    //         block.push(*el);
    //     }
    //     for group in 0..sb.block_groups_count {
    //         let start: usize = (group * 32) as usize;
    //         serial_println!(
    //             "[{}] Block address of block usage bitmap: {}",
    //             group,
    //             u32::from_le_bytes(block.as_slice()[start..(start + 4)].try_into().unwrap())
    //         );
    //         serial_println!(
    //             "[{}] Block address of inode usage bitmap: {}",
    //             group,
    //             u32::from_le_bytes(
    //                 block.as_slice()[(start + 4)..(start + 8)]
    //                     .try_into()
    //                     .unwrap()
    //             )
    //         );
    //         serial_println!(
    //             "[{}] Block address of inode table: {}",
    //             group,
    //             u32::from_le_bytes(
    //                 block.as_slice()[(start + 8)..(start + 12)]
    //                     .try_into()
    //                     .unwrap()
    //             )
    //         );
    //         serial_println!(
    //             "[{}] Free blocks count: {}",
    //             group,
    //             u16::from_le_bytes(
    //                 block.as_slice()[(start + 12)..(start + 14)]
    //                     .try_into()
    //                     .unwrap()
    //             )
    //         );
    //         serial_println!(
    //             "[{}] Free inodes count: {}",
    //             group,
    //             u16::from_le_bytes(
    //                 block.as_slice()[(start + 14)..(start + 16)]
    //                     .try_into()
    //                     .unwrap()
    //             )
    //         );
    //         serial_println!(
    //             "[{}] Directories count: {}",
    //             group,
    //             u16::from_le_bytes(
    //                 block.as_slice()[(start + 16)..(start + 18)]
    //                     .try_into()
    //                     .unwrap()
    //             )
    //         );
    //         serial_println!()
    //     }
    //     // for i in 0..block.len() {
    //     //     if i % 16 == 0 {
    //     //         serial_println!();
    //     //     }
    //     //     serial_print!("{:02x} ", block[i]);
    //     // }
    // }
}
