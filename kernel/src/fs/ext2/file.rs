use super::{find_by_path, read_1kb_block, write_1kb_block, Inode, InodeType, SUPERBLOCK};
use crate::{drivers::ata, serial_println};
use alloc::vec;
use core::cmp::min;

#[derive(Debug)]
pub enum FileError {
    NotFound,
    NotAFile,
}

pub struct File {
    inode: Inode,
    offset: usize,
}
impl File {
    pub fn from_path(path: &str) -> Result<Self, FileError> {
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
            let read_block = read_address / 512;
            let read_offset = read_address % 512;
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

    fn _old_read(&mut self, buffer: &mut [u8], size: usize) -> usize {
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

            let data = read_1kb_block(current_block, block_size as u32);
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

    pub fn read(&mut self, buffer: &mut [u8], size: usize) -> usize {
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

            let data = read_1kb_block(block, block_size as u32);

            buffer[bytes_read..(bytes_read + bytes_needed)]
                .copy_from_slice(&data[block_offset..(block_offset + bytes_needed)]);
            bytes_read += bytes_needed;
            self.offset += bytes_needed;
        }
        bytes_read
    }

    fn _old_write(&mut self, buffer: &[u8], size: usize) -> usize {
        let block_size = SUPERBLOCK.lock().get_block_size() as u32;
        let mut bytes_written = 0;

        while bytes_written < size {
            let mut current_block = self.get_block(self.offset);
            if current_block == 0 {
                if let Some(block) = self.inode.allocate_new_block() {
                    current_block = block;
                } else {
                    serial_println!("Something wrong happened at block alocation in write");
                    return bytes_written;
                }
            }

            let mut data = read_1kb_block(current_block, block_size);
            let current_block_offset = self.offset % block_size as usize;

            for pos in current_block_offset..data.len() {
                if bytes_written >= size {
                    break;
                }
                data[pos] = buffer[bytes_written];
                bytes_written += 1;
                self.offset += 1;
            }
            write_1kb_block(current_block, block_size, &data, data.len());
        }
        if self.offset >= self.inode.get_size() {
            self.inode.set_size(self.offset);
        }
        bytes_written
    }

    pub fn write(&mut self, buffer: &[u8], size: usize) -> usize {
        let block_size = SUPERBLOCK.lock().get_block_size() as u32;
        let mut bytes_written = 0;

        while bytes_written < size {
            let mut current_block = self.get_block(self.offset);
            if current_block == 0 {
                if let Some(block) = self.inode.allocate_new_block() {
                    current_block = block;
                } else {
                    serial_println!("Something wrong happened at block alocation in write");
                    return bytes_written;
                }
            }

            let block_offset = self.offset % block_size as usize;
            let left_in_block = block_size as usize - block_offset;
            let left_to_write = size - bytes_written;
            let write_size = min(left_in_block, left_to_write);

            let mut data = if write_size == block_size as usize {
                vec![0u8; block_size as usize]
            } else {
                read_1kb_block(current_block, block_size)
            };

            data[block_offset..block_offset + write_size]
                .copy_from_slice(&buffer[bytes_written..bytes_written + write_size]);

            write_1kb_block(current_block, block_size, &data, data.len());

            bytes_written += write_size;
            self.offset += write_size;
        }
        if self.offset >= self.inode.get_size() {
            self.inode.set_size(self.offset);
        }
        bytes_written
    }

    pub fn seek(&mut self, offset: usize) {
        if offset > self.inode.get_size() as usize {
            panic!("Error: seek out of bounds");
        }
        self.offset = offset;
    }
}
