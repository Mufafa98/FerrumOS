use crate::drivers::ata;
use alloc::vec::Vec;

use crate::fs::ext2::BlockGroupDescriptorTable;
use crate::fs::ext2::Superblock;

use crate::serial_println;

#[derive(Debug, PartialEq)]
pub enum InodeType {
    FIFO,
    CharacterDevice,
    Directory,
    BlockDevice,
    RegularFile,
    SymbolicLink,
    Socket,
    Unknown,
}

#[derive(Debug)]
pub enum InodeFlags {
    SecureDeletion = 0x1,
    KeepCopyWhenDeleted = 0x2,
    FileCompression = 0x4,
    SynUpdate = 0x8, // New data is imediately written to disk
    ImutableFile = 0x10,
    AppendOnly = 0x20,
    NotDump = 0x40,            // File is not included in dump command
    LastAccessNoUpdate = 0x80, // Do not update last access time
    HashIndexedDir = 0x10000,
    AFSDirectory = 0x20000,
    JournalData = 0x40000,
}

#[derive(Debug)]
#[repr(C)]
struct InodeBaseFields {
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
impl InodeBaseFields {
    fn new(buf: &[u8]) -> Self {
        let inode: InodeBaseFields = unsafe { core::ptr::read(buf.as_ptr() as *const _) };
        inode
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct Inode {
    base: InodeBaseFields,
    id: usize,
}
impl Inode {
    fn from_id_old(inode_id: usize) -> Self {
        use super::BLOCK_GROUP_DESCRIPTOR_TABLE;
        use super::SUPERBLOCK;
        // get the inode size from the superblock
        let (inode_size, block_size, inodes_per_group) = {
            let sb = SUPERBLOCK.lock();
            (
                sb.get_inode_size(),
                sb.get_block_size(),
                sb.get_inodes_per_group(),
            )
        };
        // 1. Calculate group and index
        let block_group = (inode_id - 1) / inodes_per_group;
        let index = (inode_id - 1) % inodes_per_group;
        // 2. Get the block group descriptor Calculate exact location
        let inode_table_start_block = BLOCK_GROUP_DESCRIPTOR_TABLE
            .lock()
            .get_block_group_descriptor(block_group)
            .get_inode_table_start_address();
        let byte_offset = index * inode_size;
        let containing_block = byte_offset / block_size;
        let offset_in_block = byte_offset % block_size;
        // 3. Read the inode table block
        let inode_table_offset = inode_table_start_block + containing_block;
        let read_address = inode_table_offset * block_size + offset_in_block;
        let read_block = (read_address / 512) as usize;
        let read_offset = (read_address % 512) as usize;

        let inode_base;

        let mut inode_buf = [0u8; 512];
        serial_println!(
            "Inode ID: {} Block Group: {} Index: {} Inode Table Start Block: {} Byte Offset: {} Containing Block: {} Offset in Block: {}",
            inode_id, block_group, index, inode_table_start_block, byte_offset, containing_block, offset_in_block
        );
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
            inode_base = InodeBaseFields::new(&inode_data);
        } else {
            let inode_data = &inode_buf[read_offset..read_offset + inode_size as usize];
            inode_base = InodeBaseFields::new(&inode_data);
        }
        Inode {
            base: inode_base,
            id: inode_id,
        }
    }

    fn get_block(inode_id: usize) -> u32 {
        todo!("Get block for inode {}", inode_id);
    }

    pub fn from_id(inode_id: usize) -> Self {
        use super::BLOCK_GROUP_DESCRIPTOR_TABLE;
        use super::SUPERBLOCK;
        // get the inode size from the superblock
        let (inode_size, block_size, inodes_per_group) = {
            let sb = SUPERBLOCK.lock();
            (
                sb.get_inode_size(),
                sb.get_block_size(),
                sb.get_inodes_per_group(),
            )
        };
        // 1. Calculate group and index
        let block_group = (inode_id - 1) / inodes_per_group;
        let index = (inode_id - 1) % inodes_per_group;
        // 2. Get the block group descriptor Calculate exact location
        let inode_table_start_block = BLOCK_GROUP_DESCRIPTOR_TABLE
            .lock()
            .get_block_group_descriptor(block_group)
            .get_inode_table_start_address();
        let byte_offset = index * inode_size;
        let containing_block = byte_offset / block_size;
        let offset_in_block = byte_offset % block_size;
        let inode_table_offset = inode_table_start_block + containing_block;
        // 3. Read the inode table block
        let data = super::read_1mb_block(inode_table_offset as u32, block_size as u32);
        let ndata = &data[offset_in_block..offset_in_block + inode_size];
        let inode_base = InodeBaseFields::new(&ndata);
        Inode {
            base: inode_base,
            id: inode_id,
        }
    }

    fn try_from_bytes(buf: &[u8]) -> Self {
        let inode: Inode = unsafe { core::ptr::read(buf.as_ptr() as *const _) };
        inode
    }

    pub fn get_type(&self) -> InodeType {
        let file_type = self.base.mode & 0xF000;
        match file_type {
            0x1000 => InodeType::FIFO,
            0x2000 => InodeType::CharacterDevice,
            0x4000 => InodeType::Directory,
            0x6000 => InodeType::BlockDevice,
            0x8000 => InodeType::RegularFile,
            0xA000 => InodeType::SymbolicLink,
            0xC000 => InodeType::Socket,
            _ => InodeType::Unknown,
        }
    }

    pub fn get_permissions(&self) -> u16 {
        return self.base.mode & 0x0FFF;
    }

    pub fn has_flag(&self, flag: InodeFlags) -> bool {
        serial_println!("Flag: {:?}", self);
        return (self.base.flags & flag as u32) != 0;
    }

    pub fn get_direct_block(&self, index: usize) -> u32 {
        if index < 12 {
            return self.base.direct_block_pointers[index];
        } else {
            panic!("Index out of bounds");
        }
    }

    pub fn get_direct_blocks(&self) -> &[u32] {
        return &self.base.direct_block_pointers;
    }

    pub fn get_indirect_block(&self) -> u32 {
        return self.base.slightly_indirect_block_pointer;
    }

    pub fn get_doubly_indirect_block(&self) -> u32 {
        return self.base.doubly_indirect_block_pointer;
    }

    pub fn get_triply_indirect_block(&self) -> u32 {
        return self.base.triply_indirect_block_pointer;
    }

    pub fn get_size(&self) -> usize {
        return self.base.size_low as usize;
    }
}
