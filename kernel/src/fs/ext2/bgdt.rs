use alloc::vec::Vec;

#[derive(Debug)]
#[repr(C)]
pub struct BlockGroupDescriptor {
    block_usage_bitmap: u32, // block address of block usage bitmap
    inode_usage_bitmap: u32, // block address of inode usage bitmap
    inode_table: u32,        // block address of inode table
    free_blocks_count: u16,  // free blocks count
    free_inodes_count: u16,  // free inodes count
    directories_count: u16,  // directories count
}
impl BlockGroupDescriptor {
    pub fn new(buf: &[u8]) -> Self {
        if buf.len() != 32 {
            panic!("Buffer must be 32 bytes long");
        }
        let bgd: BlockGroupDescriptor = unsafe { core::ptr::read(buf.as_ptr() as *const _) };
        bgd
    }
    pub fn get_inode_table_start_address(&self) -> usize {
        return self.inode_table as usize;
    }
    pub fn get_free_block_count(&self) -> usize {
        return self.free_blocks_count as usize;
    }
    pub fn get_b_bitmap_block(&self) -> usize {
        return self.block_usage_bitmap as usize;
    }
    unsafe fn to_bytes(&self) -> Vec<u8> {
        let raw_data = core::slice::from_raw_parts(
            (self as *const BlockGroupDescriptor) as *const u8,
            ::core::mem::size_of::<BlockGroupDescriptor>(),
        );
        let mut current_data = Vec::from(raw_data);
        while current_data.len() < 32 {
            current_data.push(0);
        }
        current_data
    }
    pub fn set_free_block_count(&mut self, new_value: u16) {
        self.free_blocks_count = new_value;
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct BlockGroupDescriptorTable {
    block_group_descriptors: Vec<BlockGroupDescriptor>,
}
impl BlockGroupDescriptorTable {
    pub fn new() -> Self {
        use crate::drivers::ata;
        let mut data: Vec<u8> = Vec::new();
        let mut buf = [0u8; 512];
        for lba in 4..6 {
            let read_result = ata::read(0, lba, &mut buf);
            if read_result.is_err() {
                panic!("Failed to read from disk");
            }
            data.extend_from_slice(&buf);
        }
        let block_group_count = super::SUPERBLOCK.lock().get_block_group_count();
        let mut bgdt = BlockGroupDescriptorTable {
            block_group_descriptors: Vec::new(),
        };
        for block in 0..block_group_count {
            let start = block * 32;
            let current_buffer = &data[start..start + 32];
            let bgd = BlockGroupDescriptor::new(current_buffer);
            bgdt.block_group_descriptors.push(bgd);
        }
        bgdt
    }

    pub fn get_block_group_descriptor_count(&self) -> usize {
        self.block_group_descriptors.len()
    }
    pub fn get_block_group_descriptor(&self, index: usize) -> &BlockGroupDescriptor {
        &self.block_group_descriptors[index]
    }
    pub fn get_block_group_descriptor_as_mut(&mut self, index: usize) -> &mut BlockGroupDescriptor {
        &mut self.block_group_descriptors[index]
    }
    unsafe fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        for bgd in &self.block_group_descriptors {
            // crate::serial_println!(
            //     "BGD: {:?} {:?} {:?}",
            //     bgd.to_bytes().len(),
            //     bgd,
            //     bgd.to_bytes()
            // );
            bytes.extend_from_slice(bgd.to_bytes().as_ref());
        }
        bytes
    }
    pub fn flush(&self) {
        use crate::serial_println;
        //TODO Remove?
        unsafe {
            use crate::drivers::ata;
            let self_data = self.to_bytes();
            let mut disk_data = Vec::with_capacity(1024);
            let mut buf = [0u8; 512];
            for i in 4..6 {
                let read_result = ata::read(0, i, &mut buf);
                if read_result.is_err() {
                    panic!("Failed to read from disk");
                }
                for j in 0..512 {
                    disk_data.push(buf[j]);
                }
            }
            let mut write_flag = false;
            for i in 0..self_data.len() {
                if self_data[i] != disk_data[i] {
                    serial_println!(
                        "Data mismatch at index {}: {} != {}",
                        i,
                        self_data[i],
                        disk_data[i]
                    );
                    disk_data[i] = self_data[i];
                    write_flag = true;
                }
            }
            if write_flag {
                let write_buf = &disk_data[0..512];
                let write_result = ata::write(0, 4, &write_buf);
                if write_result.is_err() {
                    panic!("Failed to write to disk");
                }
                let write_buf = &disk_data[512..1024];
                let write_result = ata::write(0, 5, &write_buf);
                if write_result.is_err() {
                    panic!("Failed to write to disk");
                }
                serial_println!("Block Group Descriptor Table flushed to disk");
            } else {
                serial_println!("No changes to Block Group Descriptor Table, not flushing to disk");
            }
        }
    }
}
