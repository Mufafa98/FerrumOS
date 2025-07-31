// If UB appears search for
// ---------------- Optimization ----------------

use crate::fs::ext2::{read_1kb_block, write_1kb_block, BLOCK_GROUP_DESCRIPTOR_TABLE, SUPERBLOCK};

use crate::serial_println;

#[derive(Debug, PartialEq, Clone)]
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
    indirect_block_pointer: u32,
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
    unsafe fn to_bytes(&self) -> &[u8] {
        core::slice::from_raw_parts(
            (self as *const InodeBaseFields) as *const u8,
            ::core::mem::size_of::<InodeBaseFields>(),
        )
    }
}

fn find_first_free_block() -> Option<u32> {
    use super::read_1kb_block;
    let blocks_per_group = SUPERBLOCK.lock().get_blocks_per_group();
    let first_free_block = SUPERBLOCK.lock().get_first_free_data_block() as usize;
    let bgdt = BLOCK_GROUP_DESCRIPTOR_TABLE.lock();
    let bgdt_count = bgdt.get_block_group_descriptor_count();
    for id in 0..bgdt_count {
        let bg_desc = bgdt.get_block_group_descriptor(id);
        if bg_desc.get_free_block_count() == 0 {
            continue;
        }
        let block = bg_desc.get_b_bitmap_block();
        let data = read_1kb_block(block as u32, SUPERBLOCK.lock().get_block_size() as u32);
        // serial_println!("Block address phys {:#X}", block * 1024);
        for i in 0..data.len() {
            let byte = data[i];
            if byte == 0xFF {
                continue;
            }
            for j in 0..8 {
                let control_bit = 0b1 << j;
                // Print all available blocks
                // serial_println!(
                //     "Block {} group {} byte {} bit {} data {:08b} is free? {}",
                //     (i * 8 + j) + id * blocks_per_group + first_free_block,
                //     id,
                //     i,
                //     j,
                //     byte,
                //     byte & control_bit == 0
                // );
                if byte & control_bit == 0 {
                    return Some(((i * 8 + j) + id * blocks_per_group + first_free_block) as u32);
                }
            }
        }
    }
    return None;
}

fn mark_block_as_used(block_number: usize) {
    let (block_size, blocks_per_group) = {
        let sb = SUPERBLOCK.lock();
        (sb.get_block_size(), sb.get_blocks_per_group())
    };
    let block_number = block_number - 1;
    let block_group_id = block_number / blocks_per_group;
    let block_byte = block_number % blocks_per_group / 8;
    let block_bit = block_number % blocks_per_group % 8;
    let mut bgdt = BLOCK_GROUP_DESCRIPTOR_TABLE.lock();
    let bgd = bgdt.get_block_group_descriptor_as_mut(block_group_id);
    let b_map_block = bgd.get_b_bitmap_block();
    let mut data = read_1kb_block(b_map_block as u32, block_size as u32);
    let to_modify = data[block_byte];
    let after = to_modify | (1 << block_bit);
    data[block_byte] = after;
    write_1kb_block(b_map_block as u32, block_size as u32, &data, data.len());
    let bgdt_block_update = bgd.get_free_block_count() - 1;
    let sb_block_update = SUPERBLOCK.lock().get_free_blocks() - 1;
    bgd.set_free_block_count(bgdt_block_update as u16);
    SUPERBLOCK.lock().set_free_blocks(sb_block_update);
}

fn mark_block_as_free(block_number: usize) {
    let (block_size, blocks_per_group) = {
        let sb = SUPERBLOCK.lock();
        (sb.get_block_size(), sb.get_blocks_per_group())
    };
    let block_number = block_number - 1;
    let block_group_id = block_number / blocks_per_group;
    let block_byte = block_number % blocks_per_group / 8;
    let block_bit = block_number % blocks_per_group % 8;

    let mut bgdt = BLOCK_GROUP_DESCRIPTOR_TABLE.lock();
    let bgd = bgdt.get_block_group_descriptor_as_mut(block_group_id);

    let b_map_block = bgd.get_b_bitmap_block();
    let mut data = read_1kb_block(b_map_block as u32, block_size as u32);

    let to_modify = data[block_byte];
    let after = to_modify & !(1 << block_bit);
    data[block_byte] = after;

    write_1kb_block(b_map_block as u32, block_size as u32, &data, data.len());

    let bgdt_block_update = bgd.get_free_block_count() + 1;
    let sb_block_update = SUPERBLOCK.lock().get_free_blocks() + 1;

    bgd.set_free_block_count(bgdt_block_update as u16);
    SUPERBLOCK.lock().set_free_blocks(sb_block_update);
}

fn find_first_free_inode() -> Option<u32> {
    use super::read_1kb_block;
    let inodes_per_group = SUPERBLOCK.lock().get_inodes_per_group();
    let first_free_block = SUPERBLOCK.lock().get_first_free_data_block() as usize;
    let bgdt = BLOCK_GROUP_DESCRIPTOR_TABLE.lock();
    let bgdt_count = bgdt.get_block_group_descriptor_count();
    for id in 0..bgdt_count {
        let bg_desc = bgdt.get_block_group_descriptor(id);
        if bg_desc.get_free_block_count() == 0 {
            continue;
        }
        let block = bg_desc.get_i_bitmap_block();
        let data = read_1kb_block(block as u32, SUPERBLOCK.lock().get_block_size() as u32);
        for i in 0..data.len() {
            let byte = data[i];
            if byte == 0xFF {
                continue;
            }
            for j in 0..8 {
                let control_bit = 0b1 << j;
                if byte & control_bit == 0 {
                    return Some(((i * 8 + j) + id * inodes_per_group + first_free_block) as u32);
                }
            }
        }
    }
    return None;
}

fn mark_inode_as_used(inode_number: usize) {
    let (block_size, inodes_per_group) = {
        let sb = SUPERBLOCK.lock();
        (sb.get_block_size(), sb.get_inodes_per_group())
    };
    let inode_number = inode_number - 1;
    let block_group_id = inode_number / inodes_per_group;
    let inode_byte = inode_number % inodes_per_group / 8;
    let inode_bit = inode_number % inodes_per_group % 8;

    let mut bgdt = BLOCK_GROUP_DESCRIPTOR_TABLE.lock();
    let bgd = bgdt.get_block_group_descriptor_as_mut(block_group_id);

    let i_map_block = bgd.get_i_bitmap_block();
    let mut data = read_1kb_block(i_map_block as u32, block_size as u32);

    let to_modify = data[inode_byte];
    let after = to_modify | (1 << inode_bit);
    data[inode_byte] = after;
    write_1kb_block(i_map_block as u32, block_size as u32, &data, data.len());

    let bgdt_inode_update = bgd.get_free_inode_count() - 1;
    let sb_inode_update = SUPERBLOCK.lock().get_free_inodes() - 1;
    bgd.set_free_inode_count(bgdt_inode_update as u16);
    SUPERBLOCK.lock().set_free_inodes(sb_inode_update);
}

fn mark_inode_as_free(inode_number: usize) {
    let (block_size, inodes_per_group) = {
        let sb = SUPERBLOCK.lock();
        (sb.get_block_size(), sb.get_inodes_per_group())
    };
    let inode_number = inode_number - 1;
    let block_group_id = inode_number / inodes_per_group;
    let inode_byte = inode_number % inodes_per_group / 8;
    let inode_bit = inode_number % inodes_per_group % 8;

    let mut bgdt = BLOCK_GROUP_DESCRIPTOR_TABLE.lock();
    let bgd = bgdt.get_block_group_descriptor_as_mut(block_group_id);

    let i_map_block = bgd.get_i_bitmap_block();
    let mut data = read_1kb_block(i_map_block as u32, block_size as u32);

    let to_modify = data[inode_byte];
    let after = to_modify & !(1 << inode_bit);
    serial_println!(
        "Unmarking inode {} as free. Before: {:08b} After: {:08b}",
        inode_number,
        to_modify,
        after
    );
    data[inode_byte] = after;
    write_1kb_block(i_map_block as u32, block_size as u32, &data, data.len());

    let bgdt_inode_update = bgd.get_free_inode_count() + 1;
    let sb_inode_update = SUPERBLOCK.lock().get_free_inodes() + 1;
    bgd.set_free_inode_count(bgdt_inode_update as u16);
    SUPERBLOCK.lock().set_free_inodes(sb_inode_update);
}

#[derive(Debug)]
#[repr(C)]
pub struct Inode {
    base: InodeBaseFields,
    id: usize,
    block: usize,
    offset_in_block: usize,
    need_flush: bool,
}
impl Inode {
    pub fn get_id(&self) -> usize {
        self.id
    }

    fn null() -> Self {
        Inode {
            base: InodeBaseFields {
                mode: 0,
                uid: 0,
                size_low: 0,
                last_access_time: 0,
                cretion_time: 0,
                last_modification_time: 0,
                deletion_time: 0,
                group_id: 0,
                hard_links_count: 0,
                disk_sectors_count: 0,
                flags: 0,
                os_specific: 0,
                direct_block_pointers: [0; 12],
                indirect_block_pointer: 0,
                doubly_indirect_block_pointer: 0,
                triply_indirect_block_pointer: 0,
                generation_number: 0,
                file_acl: 0,
                dir_acl: 0,
                block_address_fragment: 0,
                os_specific_2: 0,
            },
            id: 0,
            block: 0,
            offset_in_block: 0,
            need_flush: false,
        }
    }

    fn clear(&mut self) {
        self.base = InodeBaseFields {
            mode: 0,
            uid: 0,
            size_low: 0,
            last_access_time: 0,
            cretion_time: 0,
            last_modification_time: 0,
            deletion_time: 0,
            group_id: 0,
            hard_links_count: 0,
            disk_sectors_count: 0,
            flags: 0,
            os_specific: 0,
            direct_block_pointers: [0; 12],
            indirect_block_pointer: 0,
            doubly_indirect_block_pointer: 0,
            triply_indirect_block_pointer: 0,
            generation_number: 0,
            file_acl: 0,
            dir_acl: 0,
            block_address_fragment: 0,
            os_specific_2: 0,
        };
    }

    pub fn from_id(inode_id: usize) -> Self {
        if inode_id == 0 {
            return Inode::null();
        }

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
        let data = super::read_1kb_block(inode_table_offset as u32, block_size as u32);
        let ndata = &data[offset_in_block..offset_in_block + inode_size];
        let inode_base = InodeBaseFields::new(&ndata);
        Inode {
            base: inode_base,
            id: inode_id,
            block: inode_table_offset,
            offset_in_block,
            need_flush: true,
        }
    }

    pub fn from_id_no_flush(inode_id: usize) -> Self {
        let mut temp = Inode::from_id(inode_id);
        temp.need_flush = false;
        temp
    }

    fn to_bytes(&self) -> &[u8] {
        unsafe { self.base.to_bytes() }
    }

    fn alloc_address_in_direct_block(
        &mut self,
        block_to_write_in: u32,
        block_size: u32,
        data_block: u32,
    ) -> Option<u32> {
        // Read the direct block
        let mut data = read_1kb_block(block_to_write_in, block_size);
        for byte_pos in 0..data.len() / 4 {
            // Construct the new block address
            let byte_pos = byte_pos * 4;
            let block = u32::from_le_bytes([
                data[byte_pos],
                data[byte_pos + 1],
                data[byte_pos + 2],
                data[byte_pos + 3],
            ]);
            // todo!("On first alocation after indirect block allocation the write does not happen right");
            // If the block is free, mark the current data_block as used,
            // append it to the list of used blocks and write the changes to disk
            if block == 0 {
                serial_println!("Block address to write: {:?}", data);
                mark_block_as_used(data_block as usize);
                data[byte_pos] = (data_block & 0xFF) as u8;
                data[byte_pos + 1] = ((data_block >> 8) & 0xFF) as u8;
                data[byte_pos + 2] = ((data_block >> 16) & 0xFF) as u8;
                data[byte_pos + 3] = ((data_block >> 24) & 0xFF) as u8;

                write_1kb_block(block_to_write_in, block_size, &data, data.len());
                let test = read_1kb_block(block_to_write_in, block_size);
                serial_println!("Block address to write:{} {:?}", block_to_write_in, test);
                return Some(data_block);
            }
        }
        // If we haven t found any free spaces in the indirect list then return
        // None and try to look in the next list (in this case the doubly ind)
        return None;
    }

    fn alloc_address_in_indirect_block(
        &mut self,
        block_to_write_in: u32,
        block_size: u32,
        data_block: u32,
    ) -> Option<u32> {
        // Read the indirect block
        let mut direct_block = data_block;
        let mut data = read_1kb_block(block_to_write_in, block_size);
        for byte_pos in 0..data.len() / 4 {
            // Construct the new block address
            let byte_pos = byte_pos * 4;
            let mut block = u32::from_le_bytes([
                data[byte_pos],
                data[byte_pos + 1],
                data[byte_pos + 2],
                data[byte_pos + 3],
            ]);
            // ---------------- Optimization ----------------
            if byte_pos < data.len() - 4 {
                let next_block = u32::from_le_bytes([
                    data[byte_pos + 4],
                    data[byte_pos + 5],
                    data[byte_pos + 6],
                    data[byte_pos + 7],
                ]);
                if next_block != 0 {
                    continue;
                }
            }
            // ---------------- Optimization ----------------
            // If the block is free, mark the current data_block as used,
            // append it to the list of used blocks and write the changes to disk
            if block == 0 {
                mark_block_as_used(data_block as usize);
                data[byte_pos] = (data_block & 0xFF) as u8;
                data[byte_pos + 1] = ((data_block >> 8) & 0xFF) as u8;
                data[byte_pos + 2] = ((data_block >> 16) & 0xFF) as u8;
                data[byte_pos + 3] = ((data_block >> 24) & 0xFF) as u8;
                write_1kb_block(block_to_write_in, block_size, &data, data.len());
                block = data_block;
                direct_block = find_first_free_block().unwrap();
            }
            if let Some(block) = self.alloc_address_in_direct_block(block, block_size, direct_block)
            {
                return Some(block);
            }
        }
        // If we haven t found any free spaces in the dindirect list then return
        // None and try to look in the next list (in this case the thirdly ind)
        return None;
    }

    fn alloc_address_in_doubly_indirect_block(
        &mut self,
        block_to_write_in: u32,
        block_size: u32,
        data_block: u32,
    ) -> Option<u32> {
        // Read the indirect block
        let mut indirect_block = data_block;
        let mut data = read_1kb_block(block_to_write_in, block_size);
        for byte_pos in 0..data.len() / 4 {
            // Construct the new block address
            let byte_pos = byte_pos * 4;
            let mut block = u32::from_le_bytes([
                data[byte_pos],
                data[byte_pos + 1],
                data[byte_pos + 2],
                data[byte_pos + 3],
            ]);
            // ---------------- Optimization ----------------
            if byte_pos < data.len() - 4 {
                let next_block = u32::from_le_bytes([
                    data[byte_pos + 4],
                    data[byte_pos + 5],
                    data[byte_pos + 6],
                    data[byte_pos + 7],
                ]);
                if next_block != 0 {
                    continue;
                }
            }
            // ---------------- Optimization ----------------
            // If the block is free, mark the current data_block as used,
            // append it to the list of used blocks and write the changes to disk
            if block == 0 {
                mark_block_as_used(data_block as usize);
                data[byte_pos] = (data_block & 0xFF) as u8;
                data[byte_pos + 1] = ((data_block >> 8) & 0xFF) as u8;
                data[byte_pos + 2] = ((data_block >> 16) & 0xFF) as u8;
                data[byte_pos + 3] = ((data_block >> 24) & 0xFF) as u8;
                write_1kb_block(block_to_write_in, block_size, &data, data.len());
                block = data_block;
                indirect_block = find_first_free_block().unwrap();
            }
            if let Some(block) =
                self.alloc_address_in_indirect_block(block, block_size, indirect_block)
            {
                return Some(block);
            }
        }
        // If we haven t found any free spaces in the dindirect list then return
        // None and try to look in the next list (in this case the thirdly ind)
        return None;
    }

    fn alloc_on_direct_block(&mut self, data_block: u32) -> Option<u32> {
        // ---------------- Optimization ----------------
        if self.base.indirect_block_pointer != 0 {
            return None;
        }
        // ---------------- Optimization ----------------
        // Iterate trough the direct block pointers
        for i in 0..self.base.direct_block_pointers.len() {
            // if the block is 0 it means that it is a free space
            // and we can use it to alocate a new block
            if self.base.direct_block_pointers[i] == 0 {
                // mark the block as used and asign it to the
                // free space in the inode
                mark_block_as_used(data_block as usize);
                self.base.direct_block_pointers[i] = data_block;
                return Some(data_block);
            }
        }
        // if we reach this point it means that there is no free space
        // in the direct block pointers, so we need to search in the next
        // list (in this case the indirect block)
        return None;
    }

    fn alloc_on_indirect_block(&mut self, data_block: u32) -> Option<u32> {
        // ---------------- Optimization ----------------
        if self.base.doubly_indirect_block_pointer != 0 {
            return None;
        }
        // ---------------- Optimization ----------------
        // if indirect block does not exists, mark the current block as used
        // and request a new block
        let block_size = SUPERBLOCK.lock().get_block_size() as u32;
        let indirect_block = self.base.indirect_block_pointer;
        let mut direct_block = data_block;

        if indirect_block == 0 {
            self.base.indirect_block_pointer = data_block as u32;
            mark_block_as_used(data_block as usize);
            direct_block = find_first_free_block().unwrap();
        }

        self.alloc_address_in_direct_block(
            self.base.indirect_block_pointer,
            block_size,
            direct_block as u32,
        )
    }

    fn alloc_on_doubly_indirect_block(&mut self, data_block: u32) -> Option<u32> {
        // ---------------- Optimization ----------------
        if self.base.triply_indirect_block_pointer != 0 {
            return None;
        }
        // ---------------- Optimization ----------------
        // if indirect block does not exists, mark the current block as used
        // and request a new block
        let doubly_indirect_block = self.base.doubly_indirect_block_pointer;
        let block_size = SUPERBLOCK.lock().get_block_size() as u32;
        let mut indirect_block = data_block;

        if doubly_indirect_block == 0 {
            self.base.doubly_indirect_block_pointer = data_block as u32;
            mark_block_as_used(data_block as usize);
            indirect_block = find_first_free_block().unwrap();
        }
        self.alloc_address_in_indirect_block(
            self.base.doubly_indirect_block_pointer,
            block_size,
            indirect_block as u32,
        )
    }

    fn alloc_on_triply_indirect_block(&mut self, data_block: u32) -> Option<u32> {
        // if indirect block does not exists, mark the current block as used
        // and request a new block
        let triply_indirect_block = self.base.triply_indirect_block_pointer;
        let mut doubly_indirect_block = data_block;
        if triply_indirect_block == 0 {
            self.base.triply_indirect_block_pointer = data_block as u32;
            mark_block_as_used(data_block as usize);
            doubly_indirect_block = find_first_free_block().unwrap();
        }
        let block_size = SUPERBLOCK.lock().get_block_size() as u32;
        self.alloc_address_in_doubly_indirect_block(
            self.base.triply_indirect_block_pointer,
            block_size,
            doubly_indirect_block as u32,
        )
    }

    pub fn allocate_new_block(&mut self) -> Option<u32> {
        let data_block = find_first_free_block().unwrap() as u32;
        if let Some(block) = self.alloc_on_direct_block(data_block) {
            // serial_println!("Allocated on direct block: {}", data_block);
            return Some(block);
        }
        if let Some(block) = self.alloc_on_indirect_block(data_block) {
            // serial_println!("Allocated on indirect block: {}", data_block);
            return Some(block);
        }
        if let Some(block) = self.alloc_on_doubly_indirect_block(data_block) {
            // serial_println!("Allocated on doubly indirect block: {}", data_block);
            return Some(block);
        }
        if let Some(block) = self.alloc_on_triply_indirect_block(data_block) {
            // serial_println!("Allocated on triply indirect block: {}", data_block);
            return Some(block);
        }
        return None;
    }

    fn list_blocks(&self) {
        use super::read_1kb_block;
        for i in 0..12 {
            let block = self.base.direct_block_pointers[i];
            if block != 0 {
                serial_println!("Direct block {}: {}", i, block);
            }
        }
        let block = self.base.indirect_block_pointer;
        if block != 0 {
            serial_println!("Slightly indirect block: {}", block);
        }
        fn list_ind(block: u32) {
            let data = read_1kb_block(block, 1024);
            for i in 0..data.len() / 4 {
                let byte_pos = i * 4;
                let block = u32::from_le_bytes([
                    data[byte_pos],
                    data[byte_pos + 1],
                    data[byte_pos + 2],
                    data[byte_pos + 3],
                ]);
                if block != 0 {
                    serial_println!("Indirect block {}: {}", i, block);
                }
            }
        }
        fn list_dind(block: u32) {
            let data = read_1kb_block(block, 1024);
            for i in 0..data.len() / 4 {
                let byte_pos = i * 4;
                let block = u32::from_le_bytes([
                    data[byte_pos],
                    data[byte_pos + 1],
                    data[byte_pos + 2],
                    data[byte_pos + 3],
                ]);
                if block != 0 {
                    serial_println!("Indirect block {}: {}", i, block);
                    list_ind(block);
                }
            }
        }
        fn list_tind(block: u32) {
            let data = read_1kb_block(block, 1024);
            for i in 0..data.len() / 4 {
                let byte_pos = i * 4;
                let block = u32::from_le_bytes([
                    data[byte_pos],
                    data[byte_pos + 1],
                    data[byte_pos + 2],
                    data[byte_pos + 3],
                ]);
                if block != 0 {
                    serial_println!("DIndirect block {}: {}", i, block);
                    list_dind(block);
                }
            }
        }
        list_ind(block);
        let block = self.base.doubly_indirect_block_pointer;
        if block != 0 {
            serial_println!("Doubly indirect block: {}", block);
            list_ind(block);
        }
        let block = self.base.triply_indirect_block_pointer;
        if block != 0 {
            serial_println!("Triply indirect block: {}", block);
            list_tind(block);
        }
        serial_println!("End of blocks");
    }

    fn count_blocks(&self) -> u32 {
        use super::read_1kb_block;
        fn list_ind(block: u32) -> u32 {
            let mut count = 0;
            let data = read_1kb_block(block, 1024);
            for i in 0..data.len() / 4 {
                let byte_pos = i * 4;
                let block = u32::from_le_bytes([
                    data[byte_pos],
                    data[byte_pos + 1],
                    data[byte_pos + 2],
                    data[byte_pos + 3],
                ]);
                if block != 0 {
                    count += 1;
                }
            }
            count
        }
        fn list_dind(block: u32) -> u32 {
            let mut count = 0;
            let data = read_1kb_block(block, 1024);
            for i in 0..data.len() / 4 {
                let byte_pos = i * 4;
                let block = u32::from_le_bytes([
                    data[byte_pos],
                    data[byte_pos + 1],
                    data[byte_pos + 2],
                    data[byte_pos + 3],
                ]);
                if block != 0 {
                    count += 1 + list_ind(block);
                }
            }
            count
        }
        fn list_tind(block: u32) -> u32 {
            let mut count = 0;
            let data = read_1kb_block(block, 1024);
            for i in 0..data.len() / 4 {
                let byte_pos = i * 4;
                let block = u32::from_le_bytes([
                    data[byte_pos],
                    data[byte_pos + 1],
                    data[byte_pos + 2],
                    data[byte_pos + 3],
                ]);
                if block != 0 {
                    count += 1 + list_dind(block);
                }
            }
            count
        }

        let mut count = 0;
        for i in 0..12 {
            if self.base.direct_block_pointers[i] != 0 {
                count += 1;
            }
        }
        let block = self.base.indirect_block_pointer;
        if block != 0 {
            count += 1 + list_ind(block);
        }
        let block = self.base.doubly_indirect_block_pointer;
        if block != 0 {
            count += 1 + list_ind(block);
        }
        let block = self.base.triply_indirect_block_pointer;
        if block != 0 {
            count += 1 + list_tind(block);
        }
        count
    }

    pub fn flush(&self) {
        use crate::drivers::ata;
        let self_data = self.to_bytes();
        let block_size = {
            let sb = super::SUPERBLOCK.lock();
            sb.get_block_size()
        };
        let mut disk_data = super::read_1kb_block(self.block as u32, block_size as u32);
        let mut write_flag = false;
        for i in 0..self_data.len() {
            if self_data[i] != disk_data[self.offset_in_block + i] {
                // serial_println!(
                //     "Data mismatch at index {}: {} != {}",
                //     i,
                //     self_data[i],
                //     disk_data[self.offset_in_block + i]
                // );
                disk_data[self.offset_in_block + i] = self_data[i];
                write_flag = true;
            }
        }
        if write_flag {
            let current_block = {
                let sb = super::SUPERBLOCK.lock();
                (self.block * sb.get_block_size()) / 512
            };
            let write_buf = &disk_data[0..512];
            let write_result = ata::write(0, current_block as u32, &write_buf);
            if write_result.is_err() {
                panic!("Failed to write to disk");
            }
            let write_buf = &disk_data[512..1024];
            let write_result = ata::write(0, (current_block + 1).try_into().unwrap(), &write_buf);
            if write_result.is_err() {
                panic!("Failed to write to disk");
            }
            // serial_println!("Inode[{}] flushed to disk", self.id);
        } else {
            // serial_println!("No changes to inode[{}], not flushing to disk", self.id);
        }
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
        return self.base.indirect_block_pointer;
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

    pub fn set_size(&mut self, size: usize) {
        self.base.size_low = size as u32;
    }

    pub fn set_block_count(&mut self, count: u32) {
        self.base.disk_sectors_count = count;
    }

    pub fn get_block_count(&self) -> u32 {
        return self.base.disk_sectors_count;
    }

    pub fn inc_hard_links_count(&mut self) {
        self.base.hard_links_count += 1;
    }

    pub fn dec_hard_links_count(&mut self) {
        if self.base.hard_links_count == 0 {
            return;
        }
        serial_println!(
            "Decrementing hard links count for inode {}: {}",
            self.id,
            self.base.hard_links_count
        );
        self.base.hard_links_count -= 1;
        if self.base.hard_links_count == 0 {
            fn dealloc_d(block: u32) {
                if block == 0 {
                    return;
                }
                let block_size = SUPERBLOCK.lock().get_block_size() as u32;
                write_1kb_block(block, block_size, &[0; 1024], 1024);
                mark_block_as_free(block as usize);
            }

            fn dealloc_i(block: u32) {
                if block == 0 {
                    return;
                }
                let block_size = SUPERBLOCK.lock().get_block_size() as u32;
                let mut data = read_1kb_block(block, block_size);

                for i in 0..256 {
                    let block_number = u32::from_le_bytes([
                        data[i * 4],
                        data[i * 4 + 1],
                        data[i * 4 + 2],
                        data[i * 4 + 3],
                    ]);
                    dealloc_d(block_number);
                }
                write_1kb_block(block, block_size, &[0; 1024], 1024);
                mark_block_as_free(block as usize);
            }

            fn dealloc_di(block: u32) {
                if block == 0 {
                    return;
                }
                let block_size = SUPERBLOCK.lock().get_block_size() as u32;
                let mut data = read_1kb_block(block, block_size);

                for i in 0..256 {
                    let block_number = u32::from_le_bytes([
                        data[i * 4],
                        data[i * 4 + 1],
                        data[i * 4 + 2],
                        data[i * 4 + 3],
                    ]);
                    dealloc_i(block_number);
                }
                write_1kb_block(block, block_size, &[0; 1024], 1024);
                mark_block_as_free(block as usize);
            }

            fn dealloc_ti(block: u32) {
                if block == 0 {
                    return;
                }
                let block_size = SUPERBLOCK.lock().get_block_size() as u32;
                let mut data = read_1kb_block(block, block_size);

                for i in 0..256 {
                    let block_number = u32::from_le_bytes([
                        data[i * 4],
                        data[i * 4 + 1],
                        data[i * 4 + 2],
                        data[i * 4 + 3],
                    ]);
                    dealloc_di(block_number);
                }
                write_1kb_block(block, block_size, &[0; 1024], 1024);
                mark_block_as_free(block as usize);
            }

            dealloc_ti(self.get_triply_indirect_block());
            dealloc_di(self.get_doubly_indirect_block());
            dealloc_i(self.get_indirect_block());
            for block in self.get_direct_blocks() {
                dealloc_d(block.clone());
            }
            mark_inode_as_free(self.id);
            self.clear();
        }
    }
}

impl Drop for Inode {
    fn drop(&mut self) {
        if self.need_flush {
            // update blocks
            let blocks = self.count_blocks() * SUPERBLOCK.lock().get_block_size() as u32 / 512;
            self.set_block_count(blocks);
            self.flush();
            SUPERBLOCK.lock().flush();
            BLOCK_GROUP_DESCRIPTOR_TABLE.lock().flush();
        }
    }
}

pub fn build_inode(inode_type: InodeType) -> Inode {
    let mut base = InodeBaseFields {
        mode: 0,
        uid: 0,
        size_low: 0,
        last_access_time: 0,
        cretion_time: 0,
        last_modification_time: 0,
        deletion_time: 0,
        group_id: 0,
        hard_links_count: 1,
        disk_sectors_count: 0,
        flags: 0,
        os_specific: 0,
        direct_block_pointers: [0; 12],
        indirect_block_pointer: 0,
        doubly_indirect_block_pointer: 0,
        triply_indirect_block_pointer: 0,
        generation_number: 0,
        file_acl: 0,
        dir_acl: 0,
        block_address_fragment: 0,
        os_specific_2: 0,
    };
    base.mode = match inode_type {
        InodeType::FIFO => 0x1000,
        InodeType::CharacterDevice => 0x2000,
        InodeType::Directory => 0x4000,
        InodeType::BlockDevice => 0x6000,
        InodeType::RegularFile => 0x8000,
        InodeType::SymbolicLink => 0xA000,
        InodeType::Socket => 0xC000,
        _ => 0x0000,
    };
    let free_inode = find_first_free_inode().expect("No free inode found");
    let free_inode = free_inode as usize;
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
    let block_group = (free_inode - 1) / inodes_per_group;
    let index = (free_inode - 1) % inodes_per_group;
    // 2. Get the block group descriptor Calculate exact location
    let inode_table_start_block = BLOCK_GROUP_DESCRIPTOR_TABLE
        .lock()
        .get_block_group_descriptor(block_group)
        .get_inode_table_start_address();
    let byte_offset = index * inode_size;
    let containing_block = byte_offset / block_size;
    let offset_in_block = byte_offset % block_size;
    let inode_table_offset = inode_table_start_block + containing_block;

    let mut inode = Inode {
        base,
        id: free_inode,
        block: inode_table_offset,
        offset_in_block,
        need_flush: true,
    };
    inode.allocate_new_block();
    if inode_type == InodeType::Directory {
        inode.set_size(1024);
    }
    mark_inode_as_used(free_inode as usize);
    inode
}
