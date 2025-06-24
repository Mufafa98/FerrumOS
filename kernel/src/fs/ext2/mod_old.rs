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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

    fn new_dir(path: &str) -> Self {
        let trimmed_path = path.trim();

        // 0. Determine parent path and new directory name
        let (parent_path_str, dir_name_str) = {
            if let Some(idx) = trimmed_path.rfind('/') {
                let p_path = &trimmed_path[..idx];
                let f_name = &trimmed_path[idx + 1..];
                if p_path.is_empty() {
                    // Path was like "/newdir", parent is "/"
                    (String::from("/"), f_name.to_string())
                } else {
                    // Path was like "a/b/newdir" or "/a/b/newdir"
                    (p_path.to_string(), f_name.to_string())
                }
            } else {
                // Path was like "newdir", parent is "."
                (String::from("."), trimmed_path.to_string())
            }
        };

        if dir_name_str.is_empty() || dir_name_str == "." || dir_name_str == ".." {
            serial_println!("Invalid directory name: '{}'", dir_name_str);
            return DirEntry {
                inode: 0,
                rec_len: 0,
                name_len: 0,
                file_type: 0,
                name: String::new(),
                next: 0,
            };
        }

        // Check if path already exists
        if let Some(existing_entry) = find_by_path(trimmed_path) {
            serial_println!("Path {} already exists.", trimmed_path);
            // Optionally, return the existing entry if it's a directory, or an error entry
            if DirEntryType::from_u8(existing_entry.file_type) == DirEntryType::Directory {
                return existing_entry;
            } else {
                return DirEntry {
                    inode: 0,
                    rec_len: 0,
                    name_len: 0,
                    file_type: 0,
                    name: String::new(),
                    next: 0,
                };
            }
        }

        // 1. Get the parent directory's inode.
        let parent_dir_entry = match find_by_path(&parent_path_str) {
            Some(pde) => pde,
            None => {
                serial_println!("Parent directory '{}' not found.", parent_path_str);
                return DirEntry {
                    inode: 0,
                    rec_len: 0,
                    name_len: 0,
                    file_type: 0,
                    name: String::new(),
                    next: 0,
                };
            }
        };

        let mut parent_inode = Inode::from_id(parent_dir_entry.inode as usize);

        if parent_inode.get_type() != InodeType::Directory {
            serial_println!("Parent path '{}' is not a directory.", parent_path_str);
            return DirEntry {
                inode: 0,
                rec_len: 0,
                name_len: 0,
                file_type: 0,
                name: String::new(),
                next: 0,
            };
        }

        // 2. Create the inode for the new directory.
        let mut new_dir_inode = build_inode(InodeType::Directory);
        new_dir_inode.set_hard_links_count(2);

        // 3. Create "." entry for the new directory.
        let mut dot_entry = DirEntry {
            inode: new_dir_inode.get_id() as u32,
            rec_len: 0, // add_to_inode will calculate this
            name_len: 1,
            file_type: DirEntryType::Directory.to_u8(),
            name: ".".to_string(),
            next: 0,
        };

        // 4. Create ".." entry for the new directory.
        let mut dot_dot_entry = DirEntry {
            inode: parent_inode.get_id() as u32,
            rec_len: 0, // add_to_inode will calculate this
            name_len: 2,
            file_type: DirEntryType::Directory.to_u8(),
            name: "..".to_string(),
            next: 0,
        };

        // 5. Add "." and ".." entries to the new directory's data block.
        dot_entry.add_to_inode(&mut new_dir_inode);
        serial_println!(
            "Added '.' entry to new directory inode: {}",
            new_dir_inode.get_id()
        );
        dot_dot_entry.add_to_inode(&mut new_dir_inode);
        serial_println!(
            "Added '..' entry to new directory inode: {}",
            new_dir_inode.get_id()
        );
        // 6. Increment parent_inode's link count because ".." in new_dir points to it.
        parent_inode.inc_hard_links_count();

        // 7. Create the DirEntry for the new directory (to be added to the parent directory).
        let mut entry_for_parent = DirEntry {
            inode: new_dir_inode.get_id() as u32,
            rec_len: 0, // add_to_inode will calculate this
            name_len: dir_name_str.len() as u8,
            file_type: DirEntryType::Directory.to_u8(),
            name: dir_name_str,
            next: 0,
        };

        // 8. Add the new directory's entry to the parent directory.
        entry_for_parent.add_to_inode(&mut parent_inode);

        serial_println!(
            "Directory created: {} (inode {}) in parent {} (inode {})",
            entry_for_parent.name,
            new_dir_inode.get_id(),
            parent_path_str,
            parent_inode.get_id()
        );

        entry_for_parent // Return the DirEntry that was added to the parent.
    }

    fn new_file(name: &str) -> Self {
        let trimmed_path = name.trim();

        // 0. Determine parent path and new file name
        let (parent_path_str, file_name_str) = {
            if let Some(idx) = trimmed_path.rfind('/') {
                let p_path = &trimmed_path[..idx];
                let f_name = &trimmed_path[idx + 1..];
                if p_path.is_empty() {
                    // Path was like "/file", parent is "/"
                    (String::from("/"), f_name.to_string())
                } else {
                    // Path was like "a/b/file" or "/a/b/file"
                    (p_path.to_string(), f_name.to_string())
                }
            } else {
                // Path was like "file", parent is "."
                (String::from("."), trimmed_path.to_string())
            }
        };

        // Validate file_name_str
        if file_name_str.is_empty() || file_name_str == "." || file_name_str == ".." {
            serial_println!("Invalid file name: '{}'", file_name_str);
            return DirEntry {
                inode: 0,
                rec_len: 0,
                name_len: 0,
                file_type: 0,
                name: String::new(),
                next: 0,
            };
        }

        // Check if path already exists (using the full trimmed_path)
        if let Some(existing_entry) = find_by_path(trimmed_path) {
            serial_println!("File {} already exists.", trimmed_path); // Consider consistency with println! vs serial_println!
            return existing_entry; // Return the existing entry
        }

        // 1. Get the parent directory's inode.
        let parent_dir_entry = match find_by_path(&parent_path_str) {
            Some(pde) => pde,
            None => {
                serial_println!("Parent directory '{}' not found.", parent_path_str);
                return DirEntry {
                    inode: 0,
                    rec_len: 0,
                    name_len: 0,
                    file_type: 0,
                    name: String::new(),
                    next: 0,
                };
            }
        };

        let mut parent_inode = Inode::from_id(parent_dir_entry.inode as usize);

        if parent_inode.get_type() != InodeType::Directory {
            serial_println!("Parent path '{}' is not a directory.", parent_path_str);
            return DirEntry {
                inode: 0,
                rec_len: 0,
                name_len: 0,
                file_type: 0,
                name: String::new(),
                next: 0,
            };
        }

        // 2. Create the inode for the new file.
        let new_file_inode = build_inode(InodeType::RegularFile);

        // 3. Create the DirEntry for the new file.
        let mut entry_for_parent = DirEntry {
            inode: new_file_inode.get_id() as u32,
            rec_len: 0, // add_to_inode will calculate the actual on-disk rec_len
            name_len: file_name_str.len() as u8,
            file_type: DirEntryType::RegularFile.to_u8(),
            name: file_name_str.clone(),
            next: 0,
        };

        // 4. Add the new file's entry to the parent directory.
        entry_for_parent.add_to_inode(&mut parent_inode);

        serial_println!(
            "File created: {} (inode {}) in parent {} (inode {})",
            entry_for_parent.name,
            new_file_inode.get_id(),
            parent_path_str,
            parent_inode.get_id()
        );

        entry_for_parent
    }

    fn create_fs_entry(path: &str, new_inode_type: InodeType) -> Self {
        let trimmed_path = path.trim();

        // 0. Determine parent path and new entry name
        let (parent_path_str, entry_name_str) = {
            if let Some(idx) = trimmed_path.rfind('/') {
                let p_path = &trimmed_path[..idx];
                let f_name = &trimmed_path[idx + 1..];
                if p_path.is_empty() {
                    // Path was like "/entry", parent is "/"
                    (String::from("/"), f_name.to_string())
                } else {
                    // Path was like "a/b/entry" or "/a/b/entry"
                    (p_path.to_string(), f_name.to_string())
                }
            } else {
                // Path was like "entry", parent is "."
                (String::from("."), trimmed_path.to_string())
            }
        };

        // Validate entry_name_str
        if entry_name_str.is_empty() || entry_name_str == "." || entry_name_str == ".." {
            serial_println!("Invalid entry name: '{}'", entry_name_str);
            return DirEntry {
                inode: 0,
                rec_len: 0,
                name_len: 0,
                file_type: 0,
                name: String::new(),
                next: 0,
            };
        }

        let target_dir_entry_type = match new_inode_type {
            InodeType::Directory => DirEntryType::Directory,
            InodeType::RegularFile => DirEntryType::RegularFile,
            // Extend this match if other InodeTypes can be created this way
            _ => {
                serial_println!(
                    "Unsupported InodeType for create_fs_entry: {:?}",
                    new_inode_type
                );
                return DirEntry {
                    inode: 0,
                    rec_len: 0,
                    name_len: 0,
                    file_type: 0,
                    name: String::new(),
                    next: 0,
                };
            }
        };

        // Check if path already exists
        if let Some(existing_entry) = find_by_path(trimmed_path) {
            serial_println!("Path {} already exists.", trimmed_path);
            if DirEntryType::from_u8(existing_entry.file_type) == target_dir_entry_type {
                return existing_entry; // Path exists and is of the correct type
            } else {
                // Path exists but is of a different type
                serial_println!(
                    "Path {} exists but is of a different type (expected {:?}, found {:?}).",
                    trimmed_path,
                    target_dir_entry_type,
                    DirEntryType::from_u8(existing_entry.file_type)
                );
                return DirEntry {
                    inode: 0,
                    rec_len: 0,
                    name_len: 0,
                    file_type: 0,
                    name: String::new(),
                    next: 0,
                };
            }
        }

        // 1. Get the parent directory's inode.
        let parent_dir_entry_opt = find_by_path(&parent_path_str);
        let parent_dir_entry = match parent_dir_entry_opt {
            Some(pde) => pde,
            None => {
                serial_println!("Parent directory '{}' not found.", parent_path_str);
                return DirEntry {
                    inode: 0,
                    rec_len: 0,
                    name_len: 0,
                    file_type: 0,
                    name: String::new(),
                    next: 0,
                };
            }
        };

        let mut parent_inode = Inode::from_id(parent_dir_entry.inode as usize);

        if parent_inode.get_type() != InodeType::Directory {
            serial_println!("Parent path '{}' is not a directory.", parent_path_str);
            return DirEntry {
                inode: 0,
                rec_len: 0,
                name_len: 0,
                file_type: 0,
                name: String::new(),
                next: 0,
            };
        }

        // 2. Create the inode for the new entry.
        // build_inode should set need_flush=true and appropriate initial link counts
        // (1 for files, 1 or 2 for dirs depending on its internal logic before set_hard_links_count)
        let mut new_entry_inode = build_inode(new_inode_type.clone());

        // Directory-specific setup
        if new_inode_type == InodeType::Directory {
            // Ensure new directory inode has 2 links: one for its name in parent, one for its own "."
            new_entry_inode.set_hard_links_count(2);

            // Create "." entry for the new directory.
            let mut dot_entry = DirEntry {
                inode: new_entry_inode.get_id() as u32,
                rec_len: 0, // add_to_inode will calculate this
                name_len: 1,
                file_type: DirEntryType::Directory.to_u8(),
                name: ".".to_string(),
                next: 0,
            };

            // Create ".." entry for the new directory.
            let mut dot_dot_entry = DirEntry {
                inode: parent_inode.get_id() as u32, // Points to parent's inode ID
                rec_len: 0,                          // add_to_inode will calculate this
                name_len: 2,
                file_type: DirEntryType::Directory.to_u8(),
                name: "..".to_string(),
                next: 0,
            };

            // Add "." and ".." entries to the new directory's data block.
            dot_entry.add_to_inode(&mut new_entry_inode);
            serial_println!(
                "Added '.' entry to new directory inode: {}",
                new_entry_inode.get_id()
            );
            dot_dot_entry.add_to_inode(&mut new_entry_inode);
            serial_println!(
                "Added '..' entry to new directory inode: {}",
                new_entry_inode.get_id()
            );

            // Increment parent_inode's link count because ".." in the new directory points to it.
            parent_inode.inc_hard_links_count();
            // parent_inode.need_flush = true; // inc_hard_links_count should ideally handle this or it's handled by Drop
        }
        // For files, build_inode should have set link count to 1.

        // 3. Create the DirEntry for the new entry (to be added to the parent directory).
        let mut entry_for_parent = DirEntry {
            inode: new_entry_inode.get_id() as u32,
            rec_len: 0, // add_to_inode will calculate the actual on-disk rec_len
            name_len: entry_name_str.len() as u8,
            file_type: target_dir_entry_type.to_u8(),
            name: entry_name_str.clone(), // Use the parsed entry_name_str
            next: 0,                      // This is a runtime field, not for on-disk struct
        };

        // 4. Add the new entry's DirEntry to the parent directory.
        // add_to_inode should handle block allocation, rec_len updates, parent inode size, and marking parent for flush.
        entry_for_parent.add_to_inode(&mut parent_inode);

        // new_entry_inode and parent_inode should be flushed to disk via their Drop impl
        // if need_flush was set by build_inode, from_id, or subsequent modifications.

        let type_str = if new_inode_type == InodeType::Directory {
            "Directory"
        } else {
            "File"
        };
        serial_println!(
            "{} created: {} (inode {}) in parent {} (inode {})",
            type_str,
            entry_for_parent.name,
            new_entry_inode.get_id(),
            parent_path_str,
            parent_inode.get_id()
        );

        entry_for_parent
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
            // Get the block size from the superblock
            // and read the block data
            let block_size = SUPERBLOCK.lock().get_block_size() as u32;
            let mut data = read_1kb_block(block, block_size);
            // Find the last entry in the directory
            let mut entry = DirEntry::from_ptr(&data, 0);
            while entry.rec_len != 0 && entry.next < data.len() {
                entry = DirEntry::from_ptr(&data, entry.next);
            }
            let mut current_offset = 0;
            let mut next = 0;
            let mut remaining_space = data.len();
            if entry.inode != 0 {
                current_offset = data.len() - entry.rec_len as usize;
                next = current_offset + entry.size();
                remaining_space = data.len() - next;
            }

            if remaining_space < new_entry.size() || block == 0 || current_offset == 0 {
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
                serial_println!(
                    "Adding entry {} to block {} at offset {}",
                    new_entry.name,
                    block,
                    current_offset
                );
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
                let new_block = inode
                    .allocate_new_block()
                    .expect("Failed to allocate new block");
                entry_on_d(new_block, inode, self);
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
        // serial_println!("Entry: {:?}", entry);
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
    // let entry = DirEntry::new_file(path);
    let entry = DirEntry::create_fs_entry(path, InodeType::RegularFile);
    if entry.inode == 0 {
        serial_println!("Failed to create file: {}", path);
        return;
    }
    serial_println!("File created: {:?}", entry);
}

pub fn mkdir(path: &str) {
    // todo!("Implement mkdir function");
    // let entry = DirEntry::new_dir(path);
    let entry = DirEntry::create_fs_entry(path, InodeType::Directory);
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
