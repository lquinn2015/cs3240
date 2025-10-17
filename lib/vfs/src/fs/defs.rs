#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct Ext2Inode {
    pub mode: u16,
    pub uid: u16,
    pub size: u32,
    pub atime: u32,
    pub ctime: u32,
    pub mtime: u32,
    pub dtime: u32,
    pub gid: u16,
    pub links_count: u16,
    pub blocks_count: u32,
    pub flags: u32,
    pub reserved0: u32,
    pub blocks: [u32; 15],

    // ACL  OS dep
    pub gen: u32,
    pub file_acl: u32,
    pub dir_acl: u32,
    pub faddr: u32,
    pub res1: [u32; 3],
}

const EXT2_SUPER_MAGIC: u16 = 0xEF53;

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct Ext2SuperBlock {
    pub inode_count: u32,
    pub block_count: u32,
    pub r_block_count: u32,
    pub free_blocks_count: u32,
    pub free_inode_count: u32,
    pub first_data_block: u32,
    pub log_block_size: u32,
    pub log_frag_size: i32,
    pub blocks_per_group: u32,
    pub frags_per_group: u32,
    pub inodes_per_group: u32,
    pub mtime: u32,
    pub wtime: u32,
    pub mnt_count: u16,
    pub max_mnt_count: u16,
    pub magic: u16,
    pub state: u16,
    pub errors: u16,
    pub minor_rev_level: u16,
    pub lastcheck: u32,
    pub check_interval: u32,
    pub creator_os: u32,
    pub rev_level: u32,
    pub def_resuid: u16,
    pub def_resgid: u16,

    // EXT2_DYNAMIC_REV
    pub first_ino: u32,
    pub inode_size: i16,
    pub block_group_nr: u16,
    pub feature_compat: u32,
    pub feature_incompat: u32,
    pub feature_ro_compat: u32,
    pub uuid: [u8; 16],
    pub volume_name: [u8; 16],
    pub last_mounted: [u8; 64],
    pub algorithm_ussage_bitmap: u32,

    //EXT2_COMPAT_PREALLOC
    pub prealloc_blocks: u8,
    pub prealloc_dir_blocks: u8,
    pub padding1: u16,
    pub reserved: [u32; 204],
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct Ext2GroupDesc {
    pub block_bitmap_idx: u32,
    pub inode_bitmap_idx: u32,
    pub inode_table_idx: u32,
    pub free_blocks_count: u16,
    pub free_inode_count: u16,
    pub used_dirs_count: u16,
    pub pad: u16,
    pub reserved: [u32; 3],
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct Ext2DirEntry {
    pub inode: u32,
    pub rec_len: u16,
    pub name_len: u8,
    pub file_type: u8,
    // there is a unsized name field at most 256 bytes long after this
}
