use super::traits::block_device::BlockDevice;
use std::io::{self, Read, Seek, Write};

#[derive(Debug, Clone, Copy)]
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

#[derive(Debug, Clone, Copy)]
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

#[derive(Debug, Clone, Copy)]
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

#[derive(Debug, Clone, Copy)]
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

        let sb: Ext2SuperBlock = unsafe { core::mem::transmute(buf) };

        Ok(Ext2DevHandle { dev, sb })
    }

    fn read_struct<C: Copy>(&mut self, offset: u64) -> io::Result<C> {
        let mut obj = core::mem::MaybeUninit::<C>::uninit();
        let mut sbuf = core::mem::MaybeUninit::<[u8; 512]>::uninit();

        let (mut dst, mut src) = unsafe {
            (
                core::slice::from_raw_parts_mut(
                    obj.as_mut_ptr() as *mut u8,
                    core::mem::size_of::<C>(),
                ),
                core::slice::from_raw_parts_mut(sbuf.as_mut_ptr() as *mut u8, 512),
            )
        };

        let start_sector = offset / self.sector_size();
        let mut t_start = (offset - (self.sector_size() * start_sector)) as usize;

        let mut iter = 0;
        while !dst.is_empty() {
            self.read_sector(start_sector + iter, &mut src).unwrap();

            iter += 1;
            let to_read = usize::min(dst.len(), src[t_start..].len());

            let tslice = &mut src[t_start..t_start + to_read];
            t_start = 0;

            let (to_fill, rdst) = dst.split_at_mut(tslice.len());
            to_fill.copy_from_slice(&tslice);
            dst = rdst;
        }

        unsafe { Ok(obj.assume_init()) }
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
        let inode_table_addr = (my_bg_desc.inode_table_idx * block_size) as u64;
        let inode_off = (idx * inode_size) as u64;
        let off = inode_table_addr + inode_off;

        println!("inode byte addr: {off:0x}, inode_table_idx: {}, inode_table_addr: {inode_table_addr:0x}, inode_bg_off: {idx}", my_bg_desc.inode_table_idx);

        self.read_struct(off).unwrap()
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

        self.read_struct(byte_addr).unwrap()
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
