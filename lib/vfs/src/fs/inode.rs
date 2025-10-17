use super::defs::*;
use super::fs::*;
use super::group::*;

use crate::traits::block_device::BlockDevice;

pub struct Ext2InodeHandle {
    ino: u64,
    off: u64,
}

fn read_inode_data(fs: &mut Ext2DevHandle, ino: u64, offset: u64, buf: &mut [u8]) -> Result<usize, ()> {

}

fn get_inode_block(fs: &mut Ext2DirEntry, inode: Ext2InodeHandle, )
fn read_inode_block(); 


impl Ext2InodeHandle {
    pub fn create(ino: u64) -> Ext2InodeHandle {
        Ext2InodeHandle { ino, off: 0 }
    }


    pub fn read(&mut self, fs: &mut Ext2DevHandle, buf: &mut [u8]) -> Result<usize, ()> {
        let bg_num = self.ino / fs.sb.inodes_per_group as u64;
        let bg = get_block_group(fs, bg_num)?;
        let inode = get_inode(fs, self.ino)?;

        let mut sbuf = [0; 512];
        let mut dst = &mut buf[0..];
        let src = &mut sbuf[0..];

        let b2r = inode.blocks[0] as u64;

        let offset = b2r * fs.block_size();

        let start_sector = (offset) / fs.sector_size();
        let start_off = (offset - (fs.sector_size() * start_sector)) as usize;

        let mut t_start = 0;
        let mut iter = 0;

        let amt2read = dst.len();

        while !dst.is_empty() {
            fs.read_sector(start_sector + iter, src).unwrap();

            iter += 1;
            let to_read = usize::min(dst.len(), src[t_start..].len());

            let tslice = &mut src[t_start..t_start + to_read];
            t_start = 0;

            let (to_fill, rdst) = dst.split_at_mut(tslice.len());
            to_fill.copy_from_slice(&tslice);
            dst = rdst;
        }

        Ok(amt2read)
    }
}

// This should use the inode cache than read
pub fn get_inode(fs: &mut Ext2DevHandle, ino: u64) -> Result<Ext2Inode, ()> {
    Ok(read_inode(fs, ino))
}

pub fn read_inode(fs: &mut Ext2DevHandle, ino: u64) -> Ext2Inode {
    let block_size = fs.block_size();

    let gno = ino / fs.sb.inodes_per_group as u64;
    let idx = ino % fs.sb.inodes_per_group as u64;

    let my_bg_desc = read_block_group(fs, gno);
    let inode_size = core::mem::size_of::<Ext2Inode>() as u64;

    // FS block idx
    let inode_table_addr = (my_bg_desc.inode_table_idx as u64 * block_size) as u64;
    let inode_off = (idx * inode_size) as u64;
    let off = inode_table_addr + inode_off;

    println!("inode byte addr: {off:0x}, inode_table_idx: {}, inode_table_addr: {inode_table_addr:0x}, inode_bg_off: {idx}", my_bg_desc.inode_table_idx);

    fs.read_struct(off).unwrap()
}
