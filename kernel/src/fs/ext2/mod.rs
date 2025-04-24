// use core::result;
//TODO: Maybe implement some caching and better block alocation

use crate::drivers::*;
use crate::{serial_print, serial_println};
use spin::mutex::Mutex;

use alloc::string::String;
use alloc::vec::Vec;

mod superblock_data_types;

mod superblock;
use superblock::Superblock;

mod bgdt;
use bgdt::BlockGroupDescriptorTable;

mod inode;
use inode::{Inode, InodeType};

lazy_static::lazy_static!(
    static ref SUPERBLOCK: Mutex<Superblock> = Mutex::new(Superblock::new());

    static ref BLOCK_GROUP_DESCRIPTOR_TABLE: Mutex<BlockGroupDescriptorTable> =
        Mutex::new(BlockGroupDescriptorTable::new());
);

pub fn read_1mb_block(block_number: u32, block_size: u32) -> Vec<u8> {
    let mut result: Vec<u8> = Vec::new();

    let block_address = block_number * block_size;
    let block_1 = (block_address / 512);
    let buffer = &mut [0u8; 512];

    let read_result = ata::read(0, block_1, buffer);
    if read_result.is_err() {
        panic!("Failed to read from disk");
    }
    result.extend_from_slice(buffer);

    let read_result = ata::read(0, block_1 + 1, buffer);
    if read_result.is_err() {
        panic!("Failed to read from disk");
    }
    result.extend_from_slice(buffer);
    result
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

#[derive(Debug)]
#[repr(C)]
struct DirEntry {
    inode: u32,
    rec_len: u16,
    name_len: u8,
    file_type: u8,
    name: String,
    next: usize,
}
impl DirEntry {
    fn from_ptr(data: &[u8], start: usize) -> Self {
        let inode = u32::from_le_bytes([
            data[start],
            data[start + 1],
            data[start + 2],
            data[start + 3],
        ]);
        let rec_len = u16::from_le_bytes([data[start + 4], data[start + 5]]);
        let name_len = u8::from_le(data[start + 6]);
        let file_type = u8::from_le(data[start + 7]);
        let name =
            core::str::from_utf8(&data[(start + 8) as usize..(start + 8 + name_len as usize)])
                .unwrap();
        DirEntry {
            inode,
            rec_len,
            name_len,
            file_type,
            name: String::from(name),
            next: start + rec_len as usize,
        }
    }
}

// Finds a file in the root directory
fn find_file_in_dir(file_name: &str, root_inode: Inode) -> Option<DirEntry> {
    let block_size = SUPERBLOCK.lock().get_block_size() as u32;
    fn file_in_direct_block(
        block_number: u32,
        file_name: &str,
        block_size: u32,
    ) -> Option<DirEntry> {
        if block_number == 0 {
            return None;
        }
        let data = read_1mb_block(block_number, block_size);

        let mut entry = DirEntry::from_ptr(&data, 0);
        while entry.rec_len != 0 && entry.next < data.len() {
            if entry.name == file_name {
                return Some(entry);
            }
            entry = DirEntry::from_ptr(&data, entry.next);
        }
        if entry.name == file_name {
            return Some(entry);
        }
        None
    }
    fn file_in_indirect_block(
        block_number: u32,
        file_name: &str,
        block_size: u32,
    ) -> Option<DirEntry> {
        if block_number == 0 {
            return None;
        }
        let data = read_1mb_block(block_number, block_size);
        for i in 0..256 {
            let block_number = u32::from_le_bytes([
                data[i * 4],
                data[i * 4 + 1],
                data[i * 4 + 2],
                data[i * 4 + 3],
            ]);
            if let Some(entry) = file_in_direct_block(block_number, file_name, block_size) {
                return Some(entry);
            }
        }
        None
    }
    fn file_in_d_indirect_block(
        block_number: u32,
        file_name: &str,
        block_size: u32,
    ) -> Option<DirEntry> {
        if block_number == 0 {
            return None;
        }
        let data = read_1mb_block(block_number, block_size);
        for i in 0..256 {
            let block_number = u32::from_le_bytes([
                data[i * 4],
                data[i * 4 + 1],
                data[i * 4 + 2],
                data[i * 4 + 3],
            ]);
            if let Some(entry) = file_in_indirect_block(block_number, file_name, block_size) {
                return Some(entry);
            }
        }
        None
    }
    fn file_in_t_indirect_block(
        block_number: u32,
        file_name: &str,
        block_size: u32,
    ) -> Option<DirEntry> {
        if block_number == 0 {
            return None;
        }
        let data = read_1mb_block(block_number, block_size);
        for i in 0..256 {
            let block_number = u32::from_le_bytes([
                data[i * 4],
                data[i * 4 + 1],
                data[i * 4 + 2],
                data[i * 4 + 3],
            ]);
            if let Some(entry) = file_in_d_indirect_block(block_number, file_name, block_size) {
                return Some(entry);
            }
        }
        None
    }

    for direct_block in root_inode.get_direct_blocks() {
        if let Some(entry) = file_in_direct_block(*direct_block, file_name, block_size) {
            return Some(entry);
        }
    }
    if let Some(entry) =
        file_in_indirect_block(root_inode.get_indirect_block(), file_name, block_size)
    {
        return Some(entry);
    }
    if let Some(entry) = file_in_d_indirect_block(
        root_inode.get_doubly_indirect_block(),
        file_name,
        block_size,
    ) {
        return Some(entry);
    }
    if let Some(entry) = file_in_t_indirect_block(
        root_inode.get_triply_indirect_block(),
        file_name,
        block_size,
    ) {
        return Some(entry);
    }

    None
}

fn find_first_free_block() -> Option<usize> {
    let blocks_per_group = SUPERBLOCK.lock().get_blocks_per_group();
    let bgdt = BLOCK_GROUP_DESCRIPTOR_TABLE.lock();
    let bgdt_count = bgdt.get_block_group_descriptor_count();
    for id in 0..bgdt_count {
        let bg_desc = bgdt.get_block_group_descriptor(id);
        if bg_desc.get_free_block_count() == 0 {
            continue;
        }
        let block = bg_desc.get_b_bitmap_block();
        let data = read_1mb_block(block as u32, SUPERBLOCK.lock().get_block_size() as u32);
        for i in 0..data.len() {
            let byte = data[i];
            for j in 0..8 {
                let control_bit = 0b1 << j;
                if byte & control_bit == 0 {
                    return Some((i * 8 + j) + id * blocks_per_group);
                }
            }
        }
    }
    return None;
}

#[derive(Debug)]
enum FileError {
    NotFound,
    NotAFile,
}

fn find_by_path(path: &str) -> Option<DirEntry> {
    let path_parts: Vec<&str> = path.split('/').collect();
    let mut current_path = String::new();
    let mut current_inode = Inode::from_id(2);
    let mut counter = 0;

    for part in path_parts.iter() {
        if part.is_empty() {
            continue;
        }
        if current_inode.get_type() != InodeType::Directory {
            continue;
        }
        current_path.push_str(part);
        counter += 1;

        if let Some(entry) = find_file_in_dir(current_path.as_str(), current_inode) {
            current_inode = Inode::from_id(entry.inode as usize);
            current_path.clear();
            if counter == path_parts.len() {
                return Some(entry);
            }
        } else {
            serial_println!("File not found: {}", current_path);
            return None;
        }
    }
    serial_println!("File found: {}", current_path);
    None
}

struct File {
    inode: Inode,
    offset: usize,
}
impl File {
    fn from_path(path: &str) -> Result<Self, FileError> {
        let entry = find_by_path(path);
        if entry.is_none() {
            return Err(FileError::NotFound);
        }
        let entry = entry.unwrap();
        let inode = Inode::from_id(entry.inode as usize);
        if inode.get_type() != InodeType::RegularFile {
            return Err(FileError::NotAFile);
        }
        Ok(File { inode, offset: 0 })
    }

    fn get_block(&mut self, offset: usize) -> u32 {
        fn get_block_in_address(address: usize, block_number: usize) -> u32 {
            let necessary_block_offset = block_number * 4;
            let read_address = address + necessary_block_offset;
            let read_block = (read_address / 512);
            let read_offset = (read_address % 512);
            let mut inode_buf = [0u8; 512];
            let read_result = ata::read(0, read_block as u32, &mut inode_buf);
            if read_result.is_err() {
                panic!("Failed to read from disk");
            }
            let block_number = u32::from_le_bytes([
                inode_buf[read_offset],
                inode_buf[read_offset + 1],
                inode_buf[read_offset + 2],
                inode_buf[read_offset + 3],
            ]);
            return block_number;
        }

        let block_size = SUPERBLOCK.lock().get_block_size() as usize;
        let addresses_per_block = block_size / 4;
        let mut block_number = offset / block_size;
        if block_number < 12 {
            let block_number = self.inode.get_direct_block(block_number);
            return block_number;
        }
        block_number -= 12;
        if block_number < addresses_per_block {
            let indirect_block_address = self.inode.get_indirect_block() as usize * block_size;
            let block_number = get_block_in_address(indirect_block_address, block_number);
            return block_number;
        }
        block_number -= addresses_per_block;
        if block_number < addresses_per_block * addresses_per_block {
            let inidrect_block = block_number / addresses_per_block;
            let direct_block = block_number % addresses_per_block;

            let dindirect_block_address =
                self.inode.get_doubly_indirect_block() as usize * block_size;

            let indirect_block = get_block_in_address(dindirect_block_address, inidrect_block);
            let indirect_block_address = indirect_block as usize * block_size;

            let direct_block = get_block_in_address(indirect_block_address, direct_block);

            return direct_block;
        }
        block_number -= addresses_per_block * addresses_per_block;
        if block_number < addresses_per_block * addresses_per_block * addresses_per_block {
            let dindirect_block = block_number / addresses_per_block / addresses_per_block;
            let indirect_block = block_number / addresses_per_block % addresses_per_block;
            let direct_block = block_number % addresses_per_block;

            let tindirect_block_address =
                self.inode.get_triply_indirect_block() as usize * block_size;
            let dindirect_block = get_block_in_address(tindirect_block_address, dindirect_block);
            let dindirect_block_address = dindirect_block as usize * block_size;
            let indirect_block = get_block_in_address(dindirect_block_address, indirect_block);
            let indirect_block_address = indirect_block as usize * block_size;
            let direct_block = get_block_in_address(indirect_block_address, direct_block);

            return direct_block;
        }
        0
    }

    fn read(&mut self, buffer: &mut [u8], size: usize) -> usize {
        let mut bytes_read = 0;
        while bytes_read < size {
            let current_block = self.get_block(self.offset);
            if current_block == 0 {
                break;
            }
            let block_size = SUPERBLOCK.lock().get_block_size() as usize;
            // lazy version.. read the whole block
            let data = read_1mb_block(current_block, block_size as u32);
            let current_block_offset = self.offset % block_size;
            let mut current_iter_counter = 0;
            for i in current_block_offset..data.len() {
                if bytes_read >= size {
                    break;
                }
                buffer[bytes_read] = data[i];
                bytes_read += 1;
                current_iter_counter += 1;
            }
            self.offset += current_iter_counter;
        }
        bytes_read
    }

    fn seek(&mut self, offset: usize) {
        if offset > self.inode.get_size() as usize {
            panic!("Error: seek out of bounds");
        }
        self.offset = offset;
    }
}

fn offset_to_block(offset: usize, addresses_per_block: usize) {
    // let addresses_per_block = block_size;
    let mut block_number = offset / addresses_per_block;
    if block_number < 12 {
        return;
    }
    block_number -= 12;
    if block_number < addresses_per_block {
        return;
    }
    block_number -= addresses_per_block;
    if block_number < addresses_per_block * addresses_per_block {
        let inidrect_block = block_number / addresses_per_block;
        let direct_block = block_number % addresses_per_block;
        return;
    }
    block_number -= addresses_per_block * addresses_per_block;
    if block_number < addresses_per_block * addresses_per_block * addresses_per_block {
        let dindirect_block = block_number / addresses_per_block / addresses_per_block;
        let indirect_block = block_number / addresses_per_block % addresses_per_block;
        let direct_block = block_number % addresses_per_block;
        return;
    }
    serial_println!("Error: block {} is too large", offset / addresses_per_block);
}

pub fn init() {
    // BLOCK_GROUP_DESCRIPTOR_TABLE.lock().flush();
    // SUPERBLOCK.lock().changesth();
    // SUPERBLOCK.lock().flush();
    // let next_free_block = find_first_free_block();
    // serial_println!("Next free block: {:?}", next_free_block);
    // serial_println!("sss {:?}", temp);
    let test_file = File::from_path("1234");
    if test_file.is_err() {
        panic!("Error at creating file {:?}", test_file.err());
    }
    let mut test_file = test_file.unwrap();
    let mut buffer = [0u8; 512];
    loop {
        let bytes_read = test_file.read(&mut buffer, 512);
        // serial_println!("Read {} bytes", bytes_read);
        if bytes_read == 0 {
            break;
        }
        buffer_to_string(&buffer);
    }
    serial_println!()
}
