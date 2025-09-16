use super::traits::block_device::BlockDevice;
use std::{
    io::{Read, Seek, Write},
    mem::MaybeUninit,
    ops::Deref,
    ptr::copy_nonoverlapping,
};

#[derive(Debug)]
#[repr(C)]
pub struct Ext2Inode {
    mode: u16,
    uid: u16,
    size: u32,
    atime: u32,
    ctime: u32,
    mtime: u32,
    dtime: u32,
    gid: u16,
    links_count: u16,
    blocks_count: u32,
    flags: u32,
    reserved0: u32,
    blocks: [u32; 15],

    // ACL  OS dep
    gen: u32,
    file_acl: u32,
    dir_acl: u32,
    faddr: u32,
    res1: [u32; 3],
}

const EXT2_SUPER_MAGIC: u16 = 0xEF53;

#[derive(Debug)]
#[repr(C)]
pub struct Ext2SuperBlock {
    inode_count: u32,
    block_count: u32,
    r_block_count: u32,
    free_blocks_count: u32,
    free_inode_count: u32,
    first_data_block: u32,
    log_block_size: u32,
    log_frag_size: i32,
    blocks_per_group: u32,
    frags_per_group: u32,
    inodes_per_group: u32,
    mtime: u32,
    wtime: u32,
    mnt_count: u16,
    max_mnt_count: u16,
    magic: u16,
    state: u16,
    errors: u16,
    minor_rev_level: u16,
    lastcheck: u32,
    check_interval: u32,
    creator_os: u32,
    rev_level: u32,
    def_resuid: u16,
    def_resgid: u16,

    // EXT2_DYNAMIC_REV
    first_ino: u32,
    inode_size: i16,
    block_group_nr: u16,
    feature_compat: u32,
    feature_incompat: u32,
    feature_ro_compat: u32,
    uuid: [u8; 16],
    volume_name: [u8; 16],
    last_mounted: [u8; 64],
    algorithm_ussage_bitmap: u32,

    //EXT2_COMPAT_PREALLOC
    prealloc_blocks: u8,
    prealloc_dir_blocks: u8,
    padding1: u16,
    reserved: [u32; 204],
}

#[derive(Debug)]
#[repr(C)]
pub struct Ext2GroupDesc {
    block_bitmap_idx: u32,
    inode_bitmap_idx: u32,
    inode_table_idx: u32,
    free_blocks_count: u16,
    free_inode_count: u16,
    used_dirs_count: u16,
    pad: u16,
    reserved: [u32; 3],
}

#[derive(Debug)]
#[repr(C)]
pub struct Ext2DirEntry {
    inode: u32,
    rec_len: u16,
    name_len: u8,
    file_type: u8,
    unsafe_name: [u8; 255], // direct access is unsafe
}

#[derive(Debug)]
pub struct Ext2DevHandle {
    dev: std::fs::File,
    sb: Ext2SuperBlock,
}

//

impl Ext2DevHandle {
    pub fn mount(p: String) -> std::io::Result<Ext2DevHandle> {
        let mut dev = std::fs::File::open(p)?;

        let mut buf = [0u8; 1024];
        dev.seek(std::io::SeekFrom::Start(1024))?;
        dev.read(&mut buf)?;

        // TODO: bytemuck
        let sb: Ext2SuperBlock = unsafe { core::mem::transmute(buf) };

        Ok(Ext2DevHandle { dev, sb })
    }

    pub fn read_superblock(&mut self) -> &Ext2SuperBlock {
        &self.sb
    }

    pub fn read_inode(&mut self, idx: u32) -> Ext2Inode {
        let idx = idx;
        let block_size = 1024 << self.sb.log_block_size;

        let bg_number = idx / self.sb.inodes_per_group;
        let idx = idx % self.sb.inodes_per_group;

        let my_bg_desc = self.read_block_group(bg_number);
        let inode_size = core::mem::size_of::<Ext2Inode>() as u32;

        // FS block idx
        let inode_table_idx = my_bg_desc.inode_table_idx;
        let inode_block_idx = (idx * inode_size) / block_size;
        let inode_block_offs = ((idx * inode_size) % block_size) as usize;
        println!("inode_offset in block {:x}", inode_block_offs);

        let buf = *self.read_fs_block(inode_table_idx + inode_block_idx);

        println!("{:?}", &buf[inode_block_offs..]);

        let mut descr_buf = [0u8; 128];
        let descr = unsafe {
            core::ptr::copy_nonoverlapping(
                buf.as_ptr().add(inode_block_offs as usize),
                descr_buf.as_mut_ptr(),
                core::mem::size_of::<Ext2Inode>(),
            );

            core::mem::transmute(descr_buf)
        };

        descr
    }

    fn read_fs_block(&mut self, bidx: u32) -> Box<[u8; 0x1000]> {
        let mut arr: Box<[u8; 4096]> = Box::new([0u8; 4096]);

        let block_size = 1024 << self.sb.log_block_size;
        let sector_size = self.sector_size();
        let ratio = block_size / sector_size;

        assert!(block_size % sector_size == 0);

        let sector_idx = bidx as u64 * ratio;

        eprintln!(
            "read_fs_block {} which is sectors {}..{}",
            bidx,
            sector_idx,
            sector_idx + ratio
        );
        for i in 0..ratio {
            let sidx = (bidx as u64 * ratio) + i;
            let slice = &mut *arr;
            self.read_sector(
                sidx,
                &mut slice[(i * sector_size) as usize..((i + 1) * sector_size) as usize],
            )
            .unwrap();
        }

        arr
    }

    pub fn read_block_group(&mut self, idx: u32) -> Ext2GroupDesc {
        let block_size = 1024 << self.sb.log_block_size;
        let bgs = self.sb.block_count / self.sb.blocks_per_group;
        assert!(idx <= bgs);

        let desc_off = idx as u64 * core::mem::size_of::<Ext2GroupDesc>() as u64;

        let bg_offset = if (1024 + core::mem::size_of::<Ext2SuperBlock>()) < block_size {
            block_size as u64
        } else {
            todo!("sb + 1024 > block")
        };

        let byte_addr: u64 = bg_offset + desc_off;

        println!("Group descr: Byte_addr: 0x{:x}", byte_addr);

        let sector_idx: u64 = byte_addr / self.sector_size();
        let sector_off: usize = (byte_addr - sector_idx * self.sector_size()) as usize;

        let mut buf = [0u8; 512];
        self.read_sector(sector_idx, &mut buf);
        let buf = &buf[sector_off..sector_off + 32];

        let mut descr_buf = [0u8; 32];
        let descr = unsafe {
            core::ptr::copy_nonoverlapping(
                buf.as_ptr(),
                descr_buf.as_mut_ptr(),
                core::mem::size_of::<Ext2GroupDesc>(),
            );

            core::mem::transmute(descr_buf)
        };

        descr
    }
}

impl BlockDevice for Ext2DevHandle {
    fn sector_size(&self) -> u64 {
        512
    }
    fn read_sector(&mut self, n: u64, buf: &mut [u8]) -> std::io::Result<usize> {
        println!("reading sector {} into len {} buf", n, buf.len());
        self.dev
            .seek(std::io::SeekFrom::Start(n * self.sector_size()))
            .unwrap();
        self.dev.read(buf)
    }
    fn write_sector(&mut self, n: u64, buf: &[u8]) -> std::io::Result<usize> {
        self.dev
            .seek(std::io::SeekFrom::Start(n * self.sector_size()))
            .unwrap();
        self.dev.write(buf)
    }
}
