use alloc::vec;
use core::usize;

use crate::fs::ext2::inode::build_inode;
use crate::{drivers::*, print, println};
use crate::{serial_print, serial_println};
use spin::mutex::Mutex;

use alloc::string::{String, ToString};
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

#[derive(Debug, Clone, Copy)]
enum DirEntryType {
    Unknown = 0,
    RegularFile = 1,
    Directory = 2,
    CharacterDevice = 3,
    BlockDevice = 4,
    FIFO = 5,
    Socket = 6,
    Symlink = 7,
}
impl DirEntryType {
    fn from_u8(value: u8) -> Self {
        match value {
            0 => DirEntryType::Unknown,
            1 => DirEntryType::RegularFile,
            2 => DirEntryType::Directory,
            3 => DirEntryType::CharacterDevice,
            4 => DirEntryType::BlockDevice,
            5 => DirEntryType::FIFO,
            6 => DirEntryType::Socket,
            7 => DirEntryType::Symlink,
            _ => DirEntryType::Unknown,
        }
    }
    fn to_u8(&self) -> u8 {
        match self {
            DirEntryType::Unknown => 0,
            DirEntryType::RegularFile => 1,
            DirEntryType::Directory => 2,
            DirEntryType::CharacterDevice => 3,
            DirEntryType::BlockDevice => 4,
            DirEntryType::FIFO => 5,
            DirEntryType::Socket => 6,
            DirEntryType::Symlink => 7,
        }
    }
}

#[derive(Debug, Clone)]
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

    fn new(name: &str, file_type: InodeType) -> Self {
        let file_exists = find_by_path(name.trim());
        if file_exists.is_some() {
            println!("File {} already exists.", name);
            return file_exists.unwrap();
        }

        let tokens = name.split('/').collect::<Vec<&str>>();
        let file_name = tokens.last().unwrap_or(&"").trim();
        let root = if tokens.len() > 1 && !tokens[0].is_empty() {
            &tokens[..tokens.len() - 1].join("/")
        } else {
            "."
        };

        let inode = build_inode(file_type);
        let name_len = name.len() as u8;
        let rec_len = 8 + file_name.len() as u16; // 8 bytes for inode, rec_len, name_len, file_type
        let name = String::from(name);
        let mut entry = DirEntry {
            inode: inode.get_id() as u32,
            rec_len,
            name_len: file_name.len() as u8,
            file_type: DirEntryType::Directory.to_u8(), // Regular file
            name: file_name.to_string(),
            next: 0,
        };

        // Find inode of root folder
        let root_entry = find_by_path(root);
        let mut root_inode = if let Some(entry) = root_entry {
            Inode::from_id(entry.inode as usize)
        } else {
            serial_println!("Root directory not found: {}", root);
            return entry;
        };
        serial_println!("Root inode: {:?}", root_inode.get_id());

        entry.add_to_inode(&mut root_inode);

        entry
    }

    fn new_file(name: &str) -> Self {
        let file_exists = find_by_path(name.trim());
        if file_exists.is_some() {
            println!("File {} already exists.", name);
            return file_exists.unwrap();
        }

        let tokens = name.split('/').collect::<Vec<&str>>();
        let file_name = tokens.last().unwrap_or(&"").trim();
        let root = if tokens.len() > 1 && !tokens[0].is_empty() {
            &tokens[..tokens.len() - 1].join("/")
        } else {
            "."
        };

        let inode = build_inode(InodeType::RegularFile);
        let name_len = name.len() as u8;
        let rec_len = 8 + file_name.len() as u16; // 8 bytes for inode, rec_len, name_len, file_type
        let name = String::from(name);
        let mut entry = DirEntry {
            inode: inode.get_id() as u32,
            rec_len,
            name_len: file_name.len() as u8,
            file_type: DirEntryType::RegularFile.to_u8(), // Regular file
            name: file_name.to_string(),
            next: 0,
        };

        // Find inode of root folder
        let root_entry = find_by_path(root);
        let mut root_inode = if let Some(entry) = root_entry {
            Inode::from_id(entry.inode as usize)
        } else {
            serial_println!("Root directory not found: {}", root);
            return entry;
        };
        serial_println!("Root inode: {:?}", root_inode.get_id());

        entry.add_to_inode(&mut root_inode);

        entry
    }

    fn size(&self) -> usize {
        let mut size = (8 + self.name_len) as usize;
        let padding = (4 - (size % 4)) % 4;
        size += padding;
        size
    }

    fn get_aligned_bytes(&self) -> Vec<u8> {
        let mut data = Vec::new();
        data.extend_from_slice(&self.inode.to_le_bytes());
        data.extend_from_slice(&self.rec_len.to_le_bytes());
        data.push(self.name_len);
        data.push(self.file_type);
        data.extend_from_slice(self.name.as_bytes());
        // Align to 4 bytes
        let padding = 4 - (data.len() % 4);
        if padding < 4 {
            data.extend(vec![0; padding]);
        }
        data
    }

    fn add_to_inode(&mut self, inode: &mut Inode) {
        fn entry_on_d(block: u32, inode: &mut Inode, new_entry: &mut DirEntry) {
            let block_size = SUPERBLOCK.lock().get_block_size() as u32;
            let mut data = read_1kb_block(block, block_size);

            let mut entry = DirEntry::from_ptr(&data, 0);
            while entry.rec_len != 0 && entry.next < data.len() {
                entry = DirEntry::from_ptr(&data, entry.next);
            }
            let current_offset = data.len() - entry.rec_len as usize;
            let next = current_offset + entry.size();
            let remaining_space = data.len() - next;
            if remaining_space < entry.size() || block == 0 {
                let new_block = inode.allocate_new_block();
                if new_block == None {
                    serial_println!("Failed to allocate new block for entry {}", new_entry.name);
                    return;
                }
                let new_block = new_block.unwrap();
                data.fill(0);
                new_entry.rec_len = data.len() as u16;
                let new_entry_bytes = new_entry.get_aligned_bytes();
                data[0..new_entry_bytes.len()].copy_from_slice(&new_entry_bytes);
                write_1kb_block(new_block, block_size, &data, data.len());
            } else {
                data[current_offset..].fill(0);
                entry.rec_len = entry.size() as u16;
                let old_entry_bytes = entry.get_aligned_bytes();
                data[current_offset..next].copy_from_slice(&old_entry_bytes);

                new_entry.rec_len = (data.len() - next) as u16;
                let new_entry_bytes = new_entry.get_aligned_bytes();
                data[next..next + new_entry_bytes.len()].copy_from_slice(&new_entry_bytes);

                write_1kb_block(block, block_size, &data, data.len());
            }
        }

        fn entry_on_i(block: u32, inode: &mut Inode, new_entry: &mut DirEntry) {
            let block_size = SUPERBLOCK.lock().get_block_size() as u32;
            let mut data = read_1kb_block(block, block_size);

            for i in 0..255 {
                let block_number = u32::from_le_bytes([
                    data[i * 4],
                    data[i * 4 + 1],
                    data[i * 4 + 2],
                    data[i * 4 + 3],
                ]);
                let next_block = u32::from_le_bytes([
                    data[(i * 4) + 4],
                    data[(i * 4) + 5],
                    data[(i * 4) + 6],
                    data[(i * 4) + 7],
                ]);
                let block: u32;
                if i == 255 {
                    block = next_block;
                    break;
                } else if next_block == 0 {
                    block = block_number;
                    break;
                }
            }
            entry_on_d(block, inode, new_entry);
        }

        fn entry_on_db(block: u32, inode: &mut Inode, new_entry: &mut DirEntry) {
            let block_size = SUPERBLOCK.lock().get_block_size() as u32;
            let mut data = read_1kb_block(block, block_size);

            for i in 0..255 {
                let block_number = u32::from_le_bytes([
                    data[i * 4],
                    data[i * 4 + 1],
                    data[i * 4 + 2],
                    data[i * 4 + 3],
                ]);
                let next_block = u32::from_le_bytes([
                    data[(i * 4) + 4],
                    data[(i * 4) + 5],
                    data[(i * 4) + 6],
                    data[(i * 4) + 7],
                ]);
                let block: u32;
                if i == 255 {
                    block = next_block;
                    break;
                } else if next_block == 0 {
                    block = block_number;
                    break;
                }
            }
            entry_on_i(block, inode, new_entry);
        }

        fn entry_on_t(block: u32, inode: &mut Inode, new_entry: &mut DirEntry) {
            let block_size = SUPERBLOCK.lock().get_block_size() as u32;
            let mut data = read_1kb_block(block, block_size);

            for i in 0..255 {
                let block_number = u32::from_le_bytes([
                    data[i * 4],
                    data[i * 4 + 1],
                    data[i * 4 + 2],
                    data[i * 4 + 3],
                ]);
                let next_block = u32::from_le_bytes([
                    data[(i * 4) + 4],
                    data[(i * 4) + 5],
                    data[(i * 4) + 6],
                    data[(i * 4) + 7],
                ]);
                let block: u32;
                if i == 255 {
                    block = next_block;
                    break;
                } else if next_block == 0 {
                    block = block_number;
                    break;
                }
            }
            entry_on_db(block, inode, new_entry);
        }

        if inode.get_triply_indirect_block() != 0 {
            entry_on_t(inode.get_triply_indirect_block(), inode, self);
        } else if inode.get_doubly_indirect_block() != 0 {
            entry_on_d(inode.get_doubly_indirect_block(), inode, self);
        } else if inode.get_indirect_block() != 0 {
            entry_on_i(inode.get_indirect_block(), inode, self);
        } else {
            let direct_blocks = inode.get_direct_blocks();
            if direct_blocks.is_empty() {
                todo!("No direct blocks found, need to create a new block and allocate on it");
                return;
            }
            let mut last_block = direct_blocks[direct_blocks.len() - 1];
            for idx in 0..direct_blocks.len() - 1 {
                if direct_blocks[idx + 1] == 0 {
                    last_block = direct_blocks[idx];
                    break;
                }
            }
            entry_on_d(last_block, inode, self);
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

fn get_files_in_dir(path: &str) -> Vec<DirEntry> {
    let entry = find_by_path(path);
    if entry.is_none() {
        return Vec::new();
    }
    let entry = entry.unwrap();
    let inode = Inode::from_id_no_flush(entry.inode as usize);
    if inode.get_type() != InodeType::Directory {
        println!("Path {} is not a directory", path);
        return Vec::new();
    }
    let block_size = SUPERBLOCK.lock().get_block_size() as u32;
    let mut entries: Vec<DirEntry> = Vec::new();

    fn in_direct_block(block_number: u32, block_size: u32) -> Vec<DirEntry> {
        let mut entries: Vec<DirEntry> = Vec::new();
        if block_number == 0 {
            return entries;
        }
        let data = read_1kb_block(block_number, block_size);

        let mut entry = DirEntry::from_ptr(&data, 0);
        while entry.rec_len != 0 && entry.next < data.len() {
            entries.push(entry.clone());
            entry = DirEntry::from_ptr(&data, entry.next);
        }
        entries.push(entry.clone());
        entries
    }
    fn in_indirect_block(block_number: u32, block_size: u32) -> Vec<DirEntry> {
        let mut entries: Vec<DirEntry> = Vec::new();
        if block_number == 0 {
            return entries;
        }
        let data = read_1kb_block(block_number, block_size);
        for i in 0..256 {
            let block_number = u32::from_le_bytes([
                data[i * 4],
                data[i * 4 + 1],
                data[i * 4 + 2],
                data[i * 4 + 3],
            ]);
            entries.append(&mut in_direct_block(block_number, block_size));
        }
        entries
    }
    fn in_d_indirect_block(block_number: u32, block_size: u32) -> Vec<DirEntry> {
        let mut entries: Vec<DirEntry> = Vec::new();
        if block_number == 0 {
            return entries;
        }
        let data = read_1kb_block(block_number, block_size);
        for i in 0..256 {
            let block_number = u32::from_le_bytes([
                data[i * 4],
                data[i * 4 + 1],
                data[i * 4 + 2],
                data[i * 4 + 3],
            ]);
            entries.append(&mut in_indirect_block(block_number, block_size));
        }
        entries
    }
    fn in_t_indirect_block(block_number: u32, block_size: u32) -> Vec<DirEntry> {
        let mut entries: Vec<DirEntry> = Vec::new();
        if block_number == 0 {
            return entries;
        }
        let data = read_1kb_block(block_number, block_size);
        for i in 0..256 {
            let block_number = u32::from_le_bytes([
                data[i * 4],
                data[i * 4 + 1],
                data[i * 4 + 2],
                data[i * 4 + 3],
            ]);
            entries.append(&mut in_d_indirect_block(block_number, block_size));
        }
        entries
    }

    for block in inode.get_direct_blocks() {
        entries.append(&mut in_direct_block(*block, block_size));
    }
    entries.append(&mut in_indirect_block(
        inode.get_indirect_block(),
        block_size,
    ));
    entries.append(&mut in_d_indirect_block(
        inode.get_doubly_indirect_block(),
        block_size,
    ));
    entries.append(&mut in_t_indirect_block(
        inode.get_triply_indirect_block(),
        block_size,
    ));
    entries
}

pub fn ls(path: Option<&str>) {
    use alloc::format;
    let path = path.unwrap_or(".");
    let entries = get_files_in_dir(path);
    for entry in entries.iter() {
        serial_println!("Entry: {:?}", entry);
        let inode = Inode::from_id_no_flush(entry.inode as usize);
        let entry_type = match DirEntryType::from_u8(entry.file_type) {
            DirEntryType::Directory => "DIR ",
            DirEntryType::RegularFile => "FILE",
            DirEntryType::CharacterDevice => "CHAR",
            DirEntryType::BlockDevice => "BLOCK",
            DirEntryType::FIFO => "FIFO",
            DirEntryType::Socket => "SOCK",
            DirEntryType::Symlink => "LINK",
            DirEntryType::Unknown => "UNKN",
        };
        let size = inode.get_size();
        println!("{:<5} {:<20} {:>10} bytes", entry_type, entry.name, size);
    }
}

pub fn touch(path: &str) {
    let entry = DirEntry::new_file(path);
    if entry.inode == 0 {
        serial_println!("Failed to create file: {}", path);
        return;
    }
    serial_println!("File created: {:?}", entry);
}

pub fn mkdir(path: &str) {
    todo!("Implement mkdir function");
    let entry = DirEntry::new(path, InodeType::Directory);
    if entry.inode == 0 {
        serial_println!("Failed to create directory: {}", path);
        return;
    }
    serial_println!("Directory created: {:?}", entry);
}

pub fn init() {
    use inode::InodeType;
    // ls(None);
    // DirEntry::new_file("test.txt");
    // DirEntry::new_file("lorem_ipsum.txt");
    // DirEntry::new_file("test_dir/test.txt");
    // DirEntry::new_file("test_dir/test_dir2/test2.txt");
    // ls(None);
    // ls(Some("lost+found"));

    // let test_file = File::from_path("1234");
    // let lorem_ipsum = File::from_path("let");

    // if test_file.is_err() {
    //     panic!("Error at creating file {:?}", test_file.err());
    // }
    // if lorem_ipsum.is_err() {
    //     panic!("Error at creating file {:?}", lorem_ipsum.err());
    // }

    // let mut lorem_ipsum = lorem_ipsum.unwrap();
    // let mut test_file = test_file.unwrap();

    // let mut written = 0;
    // let mut readen = 0;
    // let bytes_written = 0;
    // let mut temp = 0;
    // loop {
    //     let mut buffer = [0u8; 1024];
    //     let bytes_read = lorem_ipsum.read(&mut buffer, 1024);
    //     if bytes_read == 0 {
    //         break;
    //     }
    //     let bytes_written = test_file.write(&buffer, bytes_read);

    //     written += bytes_written;
    //     readen += bytes_read;
    //     temp += 1;
    // }

    // serial_println!("\nDone writing {} bytes of {} {}\n", written, readen, temp);

    // test_file.seek(0);
    // let mut buffer = [0u8; 1024];
    // while test_file.read(&mut buffer, 1024) != 0 {
    //     for i in 0..1024 {
    //         serial_print!("{}", buffer[i] as char);
    //     }
    // }
}
