use core::cmp::min;
use core::usize;

//TODO: Implement last time write updates
use crate::{drivers::*, print};
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

fn test_read1k() {
    let data = &include_bytes!("../../../../di_128M.img");
    for block in 0..1024 {
        let read_data = read_1mb_block(block, 1024);
        for i in 0..1024 {
            if data[(block * 1024 + i) as usize] != read_data[i as usize] {
                serial_println!("Plang at {}", i);
            }
        }
    }
    serial_println!("Read test passed");
}

fn test_write1k() {
    for i in 0..1024 {
        let old_data = read_1mb_block(i, 1024);
        let garbage = [12u8; 1024];
        write_1mb_block(i, 1024, &garbage, garbage.len());
        let result = read_1mb_block(i, 1024);
        for j in 0..1024 {
            if garbage[j] != result[j] {
                serial_println!("Plang at {}", j);
            }
        }
        write_1mb_block(i, 1024, &old_data, old_data.len());
        let result = read_1mb_block(i, 1024);
        for j in 0..1024 {
            if old_data[j] != result[j] {
                serial_println!("Plang at {} after final write", j);
            }
        }
    }
    serial_println!("Write test passed");
}

fn test_read() {
    let data = &include_bytes!("../../../../let");
    let mut test_file = File::from_path("let").unwrap();
    let mut counter = 0;
    loop {
        let mut buffer = [0u8; 1024];
        let bytes_read = test_file.read(&mut buffer, 1024);
        if bytes_read == 0 {
            break;
        }
        // buffer_to_string(&buffer);
        for i in 0..bytes_read {
            if data[counter] != buffer[i] {
                serial_println!(
                    "Plang at {} {} {}",
                    counter,
                    data[counter] as char,
                    buffer[i] as char
                );
            }
            counter += 1;
        }
    }
    serial_println!("Read test passed");
}

fn print_hex(buffer: &[u8]) {
    for i in 0..buffer.len() {
        if i % 16 == 0 {
            serial_println!();
        }
        serial_print!("{:02X} ", buffer[i]);
    }
}

pub fn read_1mb_block(block_number: u32, block_size: u32) -> Vec<u8> {
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

pub fn write_1mb_block(block_number: u32, block_size: u32, buffer: &[u8], size: usize) {
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

#[derive(Debug)]
enum FileError {
    NotFound,
    NotAFile,
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
                serial_println!("Failed to read from disk . Error {:?}", read_result.err());
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

    fn old_read(&mut self, buffer: &mut [u8], size: usize) -> usize {
        let mut bytes_read = 0;
        let start_offset = self.offset;
        let block_size = SUPERBLOCK.lock().get_block_size();
        let file_size = self.inode.get_size();
        loop {
            if bytes_read >= size {
                break;
            }
            if self.offset >= file_size {
                break;
            }
            let current_block = self.get_block(self.offset);
            if current_block == 0 {
                break;
            }

            let data = read_1mb_block(current_block, block_size as u32);
            let current_block_offset = self.offset % block_size;
            let mut current_iter_counter = 0;

            for i in current_block_offset..data.len() {
                if start_offset + bytes_read >= file_size || bytes_read >= size || data[i] == 0 {
                    return bytes_read;
                }
                buffer[bytes_read] = data[i];
                bytes_read += 1;
                current_iter_counter += 1;
            }
            self.offset += current_iter_counter;
        }
        bytes_read
    }

    fn read(&mut self, buffer: &mut [u8], size: usize) -> usize {
        let block_size = SUPERBLOCK.lock().get_block_size();
        let file_size = self.inode.get_size();

        let actual_size = min(buffer.len(), size);
        let mut bytes_read = 0;

        while bytes_read < actual_size && self.offset < file_size {
            let block = self.get_block(self.offset);
            let block_offset = self.offset % block_size;
            let bytes_avail_in_block = block_size - block_offset;
            let bytes_until_eof = file_size - self.offset;
            let bytes_needed = min(
                actual_size - bytes_read,
                min(bytes_avail_in_block, bytes_until_eof),
            );

            if block == 0 {
                break;
            }

            let data = read_1mb_block(block, block_size as u32);

            buffer[bytes_read..(bytes_read + bytes_needed)]
                .copy_from_slice(&data[block_offset..(block_offset + bytes_needed)]);
            bytes_read += bytes_needed;
            self.offset += bytes_needed;
        }
        bytes_read
    }

    fn old_write(&mut self, buffer: &[u8], size: usize) -> usize {
        let block_size = SUPERBLOCK.lock().get_block_size() as u32;
        let mut bytes_written = 0;

        while bytes_written < size {
            let mut current_block = self.get_block(self.offset);
            if current_block == 0 {
                serial_println!("\nNeed to allocate new block");
                if let Some(block) = self.inode.allocate_new_block() {
                    current_block = block;
                } else {
                    serial_println!("Something wrong happened at block alocation in write");
                    return bytes_written;
                }
            }

            let mut data = read_1mb_block(current_block, block_size);
            let current_block_offset = self.offset % block_size as usize;

            for pos in current_block_offset..data.len() {
                if bytes_written >= size {
                    break;
                }
                data[pos] = buffer[bytes_written];
                bytes_written += 1;
                self.offset += 1;
            }
            // serial_println!(
            //     "\nWriting {} bytes to block {} with offset {}",
            //     bytes_written,
            //     current_block,
            //     self.offset
            // );
            // buffer_to_string(&data);
            // write_1mb_block(current_block, block_size, &data, data.len());
        }
        if self.offset >= self.inode.get_size() {
            self.inode.set_size(self.offset);
        }
        bytes_written
    }

    fn write(&mut self, buffer: &[u8], size: usize) -> usize {
        //NOTE: MISSING BLOCK??????
        let block_size = SUPERBLOCK.lock().get_block_size() as u32;
        let mut bytes_written = 0;
        let mut temp = 0;

        while bytes_written < size {
            let mut current_block = self.get_block(self.offset);
            if current_block == 0 {
                serial_println!("\nNeed to allocate new block");
                if let Some(block) = self.inode.allocate_new_block() {
                    current_block = block;
                } else {
                    serial_println!("Something wrong happened at block alocation in write");
                    return bytes_written;
                }
            }

            let mut data = read_1mb_block(current_block, block_size);
            let current_block_offset = self.offset % block_size as usize;

            for pos in current_block_offset..data.len() {
                if bytes_written >= size {
                    break;
                }
                data[pos] = buffer[bytes_written];
                bytes_written += 1;
                self.offset += 1;
            }
            temp += 1;
            serial_println!("Current Block {}", current_block);
            write_1mb_block(current_block, block_size, &data, data.len());
        }
        if self.offset >= self.inode.get_size() {
            self.inode.set_size(self.offset);
        }
        // bytes_written
        temp as usize
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
//NOTE: asd
pub fn init() {
    // let tmp = Inode::from_id(12);
    // tmp.list_blocks();
    // let tmp = Inode::from_id(13);
    // tmp.list_blocks();
    // return;
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
        // buffer_to_string(&buffer);
        // serial_println!("Read {} bytes", bytes_read);
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
