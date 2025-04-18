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
    pub fn try_from_bytes(buf: &[u8]) -> Self {
        let bgd: BlockGroupDescriptor = unsafe { core::ptr::read(buf.as_ptr() as *const _) };
        bgd
    }
    pub fn get_inode_table_start_address(&self) -> usize {
        return self.inode_table as usize;
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct BlockGroupDescriptorTable {
    block_group_descriptors: Vec<BlockGroupDescriptor>,
}
impl BlockGroupDescriptorTable {
    pub fn try_from_bytes(buf: &[u8], block_count: usize) -> Self {
        let mut bgdt: BlockGroupDescriptorTable = BlockGroupDescriptorTable {
            block_group_descriptors: Vec::new(),
        };
        for block in 0..block_count {
            let start = block * 32;
            let bgd: BlockGroupDescriptor =
                BlockGroupDescriptor::try_from_bytes(&buf[start as usize..start as usize + 32]);
            bgdt.block_group_descriptors.push(bgd);
        }
        bgdt
    }
    pub fn get_block_group_descriptor(&self, index: usize) -> &BlockGroupDescriptor {
        &self.block_group_descriptors[index]
    }
}
