// use core::result;

use crate::drivers::*;
// use crate::io::serial;
use crate::{serial_print, serial_println};

use alloc::string::String;
use alloc::vec::Vec;

mod superblock_data_types;

mod superblock;
use superblock::Superblock;

mod bgdt;
use bgdt::BlockGroupDescriptorTable;

#[derive(Debug)]
#[repr(C)]
struct Inode {
    mode: u16,
    uid: u16,
    size_low: u32,
    last_access_time: u32,
    cretion_time: u32,
    last_modification_time: u32,
    deletion_time: u32,
    group_id: u16,
    hard_links_count: u16,
    disk_sectors_count: u32,
    flags: u32,
    os_specific: u32,
    direct_block_pointers: [u32; 12],
    slightly_indirect_block_pointer: u32,
    doubly_indirect_block_pointer: u32,
    triply_indirect_block_pointer: u32,
    generation_number: u32,
    file_acl: u32,
    dir_acl: u32,
    block_address_fragment: u32,
    os_specific_2: u32,
}
impl Inode {
    fn from_id(inode_id: usize, sb: &Superblock, bgdt: &BlockGroupDescriptorTable) -> Self {
        // get he inode size from the superblock
        // TO DO use expect instead of unwrap
        let inode_size = sb.get_inode_size();
        let block_size = (1024 << sb.get_block_size()) as usize;
        // 1. Calculate group and index
        let block_group = (inode_id - 1) / sb.get_inodes_per_group();
        let index = (inode_id - 1) % sb.get_inodes_per_group();
        // 2. Get the block group descriptor
        let bg_desc = &bgdt.get_block_group_descriptor(block_group);
        // 3. Calculate exact location
        let inode_table_start_block = bg_desc.get_inode_table_start_address();
        let byte_offset = index * inode_size;
        let containing_block = byte_offset / block_size;
        let offset_in_block = byte_offset % block_size;
        // 4. Read the inode table block
        let inode_table_offset = inode_table_start_block + containing_block;
        let read_address = inode_table_offset * block_size + offset_in_block;
        let read_block = (read_address / 512) as usize;
        let read_offset = (read_address % 512) as usize;

        let inode_struct;

        let mut inode_buf = [0u8; 512];
        let read_result = ata::read(0, read_block as u32, &mut inode_buf);
        if read_result.is_err() {
            panic!("Failed to read from disk");
        }
        if read_offset + inode_size as usize > 512 {
            panic!("Works? Inode size is larger than block size {}", inode_id);
            let mut inode_buf_ext = [0u8; 512];
            read_result = ata::read(0, (read_block + 1) as u32, &mut inode_buf_ext);
            if read_result.is_err() {
                panic!("Failed to read from disk");
            }
            let inode_data_1 = &inode_buf[read_offset..];
            let inode_data_2 = &inode_buf_ext[0..(inode_size as usize - 512 - read_offset)];
            let mut inode_data = Vec::new();
            inode_data.extend_from_slice(inode_data_1);
            inode_data.extend_from_slice(inode_data_2);
            inode_struct = Inode::try_from_bytes(&inode_data);
        } else {
            let inode_data = &inode_buf[read_offset..read_offset + inode_size as usize];
            inode_struct = Inode::try_from_bytes(&inode_data);
        }
        inode_struct
    }
    
    fn try_from_bytes(buf: &[u8]) -> Self {
        let inode: Inode = unsafe { core::ptr::read(buf.as_ptr() as *const _) };
        inode
    }

    fn get_type(&self) -> &str {
        let file_type = self.mode & 0xF000;
        match file_type {
            0x1000 => "FIFO",
            0x2000 => "Character device",
            0x4000 => "Directory",
            0x6000 => "Block device",
            0x8000 => "Regular file",
            0xA000 => "Symbolic link",
            0xC000 => "Unix socket",
            _ => "Unknown",
        }
    }

    fn get_permissions(&self) -> u16 {
        return self.mode & 0x0FFF;
    }
}

fn read_1mb_block(block_number: u32, block_size: u32) -> Vec<u8> {
    let mut result: Vec<u8> = Vec::new();

    let block_address = block_number * block_size;
    let block_1 = (block_address / 512) as usize;
    let buffer = &mut [0u8; 512];

    let read_result = ata::read(0, block_1 as u32, buffer);
    if read_result.is_err() {
        panic!("Failed to read from disk");
    }
    result.extend_from_slice(buffer);

    let read_result = ata::read(0, block_1 as u32 + 1, buffer);
    if read_result.is_err() {
        panic!("Failed to read from disk");
    }
    result.extend_from_slice(buffer);
    result
}

fn read_direct_block_pointer(block_number: u32, block_size: u32) -> u32 {
    let buffer = read_1mb_block(block_number, block_size);
    // buffer_to_string(&buffer);
    return buffer.len() as u32;
}

fn read_indirect_block(block_number: u32, block_size: u32) -> u32 {
    let result: Vec<u8> = read_1mb_block(block_number, block_size);
    let mut counter = 0;
    for i in 0..256 {
        let block_number = u32::from_le_bytes([
            result[i * 4],
            result[i * 4 + 1],
            result[i * 4 + 2],
            result[i * 4 + 3],
        ]);
        if block_number == 0 {
            //TODO another implementation?
            continue;
        }
        let size = read_direct_block_pointer(block_number, block_size);
        counter += size;
    }
    return counter;
    // serial_println!(
    //     "Indirect block number: {} with {} blocks",
    //     block_number,
    //     counter
    // );
}

fn read_dindirect_block(block_number: u32, block_size: u32) -> u32 {
    serial_println!("Dindirect block number: {}", block_number);
    let result: Vec<u8> = read_1mb_block(block_number, block_size);
    let mut counter = 0;
    for i in 0..256 {
        let block_number = u32::from_le_bytes([
            result[i * 4],
            result[i * 4 + 1],
            result[i * 4 + 2],
            result[i * 4 + 3],
        ]);
        if block_number == 0 {
            //TODO another implementation?
            continue;
        }
        // serial_println!("indirect block number: {}", block_number);
        let size = read_indirect_block(block_number, block_size);
        counter += size;
    }
    return counter;
}

fn read_tindirect_block(block_number: u32, block_size: u32) -> u32 {
    let result: Vec<u8> = read_1mb_block(block_number, block_size);
    let mut counter = 0;
    for i in 0..256 {
        let block_number = u32::from_le_bytes([
            result[i * 4],
            result[i * 4 + 1],
            result[i * 4 + 2],
            result[i * 4 + 3],
        ]);
        // serial_println!("Dindirect block number: {}", block_number);
        if block_number == 0 {
            //TODO another implementation?
            continue;
        }
        let size = read_dindirect_block(block_number, block_size);
        counter += size;
    }
    return counter;
}

fn buffer_to_string(buffer: &[u8]) -> String {
    let string = String::new();
    for i in 0..buffer.len() {
        if buffer[i] == 0 {
            continue;
        }
        // string.push(buffer[i] as char);
        serial_print!("{}", buffer[i] as char);
    }
    string
}

fn read_inode_data(inode_struct: &Inode, block_size: u32) {
    let mut total_size = 0;
    for block in inode_struct.direct_block_pointers {
        if block == 0 {
            //TODO: If the fs supports sparse blocks, implement acordingly
            continue;
        }
        let size = read_direct_block_pointer(block, 1024);
        total_size += size;
        // buffer_to_string(&data);
    }
    if inode_struct.slightly_indirect_block_pointer != 0 {
        total_size += read_indirect_block(inode_struct.slightly_indirect_block_pointer, block_size);
    }
    if inode_struct.doubly_indirect_block_pointer != 0 {
        total_size += read_dindirect_block(inode_struct.doubly_indirect_block_pointer, block_size);
    }
    if inode_struct.triply_indirect_block_pointer != 0 {
        total_size += read_tindirect_block(inode_struct.triply_indirect_block_pointer, block_size);
    }
    serial_println!(
        "Total size of inode {:?} is {} bytes",
        inode_struct,
        total_size
    );
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

    let bgdt = BlockGroupDescriptorTable::try_from_bytes(&data, sb.get_block_group_count());
    // let inode_struct = get_inode(16, sb, bgdt);
    // let inode_struct = get_inode(16, &sb, &bgdt);
    let inode_struct = Inode::from_id(16, &sb, &bgdt);
    read_inode_data(&inode_struct, 1024 << sb.get_block_size());
}
