use core::usize;

use crate::drivers::*;
use crate::{serial_print, serial_println};
use spin::mutex::Mutex;

use alloc::string::String;
use alloc::vec::Vec;

mod superblock_data_types;

mod bgdt;
mod file;
mod inode;
mod superblock;

use bgdt::BlockGroupDescriptorTable;
use file::File;
use inode::{Inode, InodeType};
use superblock::Superblock;

lazy_static::lazy_static!(
    static ref SUPERBLOCK: Mutex<Superblock> = Mutex::new(Superblock::new());

    static ref BLOCK_GROUP_DESCRIPTOR_TABLE: Mutex<BlockGroupDescriptorTable> =
        Mutex::new(BlockGroupDescriptorTable::new());
);

fn print_hex(buffer: &[u8]) {
    for i in 0..buffer.len() {
        if i % 16 == 0 {
            serial_println!();
        }
        serial_print!("{:02X} ", buffer[i]);
    }
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

pub fn read_1kb_block(block_number: u32, block_size: u32) -> Vec<u8> {
    let mut result: Vec<u8> = Vec::new();

    let block_address = block_number * block_size;
    let block_1 = block_address / 512;
    let block_2 = block_1 + 1;

    let buffer = &mut [0u8; 512];
    let read_result = ata::read(0, block_1, buffer);
    if read_result.is_err() {
        serial_println!("Failed to read from disk. Error {:?}", read_result.err());
    }
    result.extend_from_slice(buffer);

    let read_result = ata::read(0, block_2, buffer);
    if read_result.is_err() {
        serial_println!("Failed to read from disk. Error {:?}", read_result.err());
    }
    result.extend_from_slice(buffer);
    result
}

pub fn write_1kb_block(block_number: u32, block_size: u32, buffer: &[u8], size: usize) {
    if size != 1024 {
        serial_println!("Buffer len should be 1024");
        return;
    }
    let block_address = block_number * block_size;
    let block_1 = block_address / 512;
    let block_2 = block_1 + 1;

    let data_1 = &buffer[0..512];
    let data_2 = &buffer[512..1024];

    let write_result = ata::write(0, block_1, &data_1);
    if write_result.is_err() {
        serial_println!(
            "Failed to write first part of data on disk {:?}",
            write_result.err()
        );
    }
    let write_result = ata::write(0, block_2, &data_2);
    if write_result.is_err() {
        serial_println!(
            "Failed to write second part of data on disk {:?}",
            write_result.err()
        );
    }
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
            core::str::from_utf8(&data[(start + 8)..(start + 8 + name_len as usize)]).unwrap();
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
        let data = read_1kb_block(block_number, block_size);

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
        let data = read_1kb_block(block_number, block_size);
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
        let data = read_1kb_block(block_number, block_size);
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
        let data = read_1kb_block(block_number, block_size);
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

fn find_by_path(path: &str) -> Option<DirEntry> {
    let path_parts: Vec<&str> = path.split('/').collect();
    let mut current_path = String::new();
    let mut current_inode = Inode::from_id_no_flush(2);
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
            current_inode = Inode::from_id_no_flush(entry.inode as usize);
            current_path.clear();
            if counter == path_parts.len() {
                return Some(entry);
            }
        } else {
            serial_println!("File not found: {}", current_path);
            return None;
        }
    }
    serial_println!("File found: {}", path);
    None

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
    let test_file = File::from_path("1234");
    let lorem_ipsum = File::from_path("let");

    if test_file.is_err() {
        panic!("Error at creating file {:?}", test_file.err());
    }
    if lorem_ipsum.is_err() {
        panic!("Error at creating file {:?}", lorem_ipsum.err());
    }

    let mut lorem_ipsum = lorem_ipsum.unwrap();
    let mut test_file = test_file.unwrap();

    let mut written = 0;
    let mut readen = 0;
    let bytes_written = 0;
    let mut temp = 0;
    loop {
        let mut buffer = [0u8; 1024];
        let bytes_read = lorem_ipsum.read(&mut buffer, 1024);
        if bytes_read == 0 {
            break;
        }
        let bytes_written = test_file.write(&buffer, bytes_read);

        written += bytes_written;
        readen += bytes_read;
        temp += 1;
    }

    serial_println!("\nDone writing {} bytes of {} {}\n", written, readen, temp);

    test_file.seek(0);
    let mut buffer = [0u8; 1024];
    while test_file.read(&mut buffer, 1024) != 0 {
        for i in 0..1024 {
            serial_print!("{}", buffer[i] as char);
        }
    }
}
