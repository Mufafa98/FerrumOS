use super::superblock_data_types as sbdt;

#[derive(Debug)]
#[repr(C)]
pub struct SuperblockExtendedFields {
    first_n_r_inode: u32,             // first non-reserved inode
    inode_size: u16,                  // size of an inode structure in bytes
    block_group_number: u16,          // block group number of the superblock
    optional_features: sbdt::Feature, // optional features supported
    required_features: sbdt::Feature, // required features supported
    readonly_features: sbdt::Feature, // features that if not present, the drive should be read-only
    fs_id: [u8; 16],                  // filesystem ID
    volume_name: [u8; 16],            // volume name
    last_mounted: [u8; 64],           // last mounted path
    compression: u32,                 // compression algorithm used
    prealloc_blocks_f: u8,            // number of blocks to preallocate for
    prealloc_blocks_d: u8,            // number of directories to preallocate for
    unused: u16,                      // unused
    journal_id: [u8; 16],             // journal ID
    journal_inum: u32,                // journal inode number
    journal_dev: u32,                 // journal device number
    head_orphan: u32,                 // head of orphan list
}
impl SuperblockExtendedFields {
    pub fn try_from_bytes(buf: &[u8]) -> Self {
        let temp: SuperblockExtendedFields = unsafe { core::ptr::read(buf.as_ptr() as *const _) };
        temp
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct Superblock {
    inodes_count: u32,                                 // total number of inodes
    blocks_count: u32,                                 // total number of blocks
    reserved_blocks_count: u32,                        // reserved for superuser
    free_blocks_count: u32,                            // unallocated blocks
    free_inodes_count: u32,                            // unallocated inodes
    superblock_block_number: u32,                      // block number of the superblock
    block_size: u32,                                   // size of a block in bytes
    fragment_size: u32,                                // size of a fragment in bytes
    blocks_per_group: u32,                             // number of blocks in a block group
    fragments_per_group: u32,                          // number of fragments in a block group
    inodes_per_group: u32,                             // number of inodes in a block group
    last_mount_time: u32,                              // time of last mount
    last_write_time: u32,                              // time of last write
    mount_count: u16,                                  // number of mounts since last check
    max_mount_count: u16,                              // max mounts before check
    signature: u16,       // signature of the filesystem (should be 0xEF53 for ext2)
    fs_state: u16,        // state of the filesystem (0 = clean, 1 = errors)
    error_handling: u16,  // error handling (0 = ignore, 1 = remount read-only, 2 = panic)
    minor_version: u16,   // minor version of the filesystem
    last_check_time: u32, // time of last check
    check_interval: u32,  // max time between checks
    creator_os: u32,      // OS that created the filesystem
    major_version: u32,   // major version of the filesystem
    uid_reserved: u16,    // uid that can use reserved blocks
    gid_reserved: u16,    // gid that can use reserved blocks
    extended_fields: Option<SuperblockExtendedFields>, // extended fields
    block_groups_count: u32, // number of block groups
}
impl Superblock {
    pub fn try_from_bytes(buf: &[u8]) -> Self {
        let mut sb: Superblock = unsafe { core::ptr::read(buf.as_ptr() as *const _) };
        let temp_ext: SuperblockExtendedFields =
            SuperblockExtendedFields::try_from_bytes(&buf[84..236]);
        sb.extended_fields = Some(temp_ext);
        let met_1 = sb.blocks_count / sb.blocks_per_group;
        let met_2 = sb.inodes_count / sb.inodes_per_group;
        if met_1 != met_2 {
            panic!("Inconsistent number of block groups");
        }
        sb.block_groups_count = met_1;
        sb
    }
    pub fn get_block_size(&self) -> usize {
        return self.block_size as usize;
    }
    pub fn get_inode_size(&self) -> usize {
        return self.extended_fields.as_ref().unwrap().inode_size as usize;
    }
    pub fn get_inodes_per_group(&self) -> usize {
        return self.inodes_per_group as usize;
    }
    pub fn get_block_group_count(&self) -> usize {
        return self.block_groups_count as usize;
    }
}
