use super::defs::*;
use super::fs::*;
use super::group::*;

use crate::fs::error::FsError;
use crate::traits::block_device::BlockDevice;

pub struct Ext2InodeHandle {
    ino: u64,
    off: u64,
}

fn read_inode_data(
    fs: &mut Ext2DevHandle,
    ino: u64,
    offset: u64,
    buf: &mut [u8],
) -> Result<usize, ()> {
    Err(())
}

//fn get_inode_block(fs: &mut Ext2DirEntry, inode: Ext2InodeHandle, )
//fn read_inode_block();

impl Ext2InodeHandle {
    pub fn create(ino: u64) -> Ext2InodeHandle {
        Ext2InodeHandle { ino, off: 0 }
    }

    pub fn read(&mut self, fs: &mut Ext2DevHandle, buf: &mut [u8]) -> Result<usize, FsError> {
        let inode = get_inode(fs, self.ino)?;

        let mut sbuf = [0; 512];
        let mut dst = &mut buf[0..];
        let src = &mut sbuf[0..];

        let b2r = inode.blocks[0] as u64;

        let offset = b2r * fs.block_size();

        let start_sector = (offset) / fs.sector_size();

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
pub fn get_inode(fs: &mut Ext2DevHandle, ino: u64) -> Result<Ext2Inode, FsError> {
    Ok(read_inode(fs, ino)?)
}

#[derive(PartialEq, Eq)]
pub enum InodeType {
    Unknown = 0,
    Fifo = 1,
    CharDev = 2,
    Dir = 4,
    BlockDev = 6,
    RegularFile = 8,
    SymLink = 0xa,
    UnixSocket = 0xc,
}

pub fn get_inode_type(inode: &Ext2Inode) -> InodeType {
    use InodeType::*;
    match (inode.mode >> 12) & 0xf {
        1 => Fifo,
        2 => CharDev,
        4 => Dir,
        6 => BlockDev,
        8 => RegularFile,
        0xa => SymLink,
        0xc => UnixSocket,
        _ => Unknown,
    }
}

pub fn read_inode(fs: &mut Ext2DevHandle, ino: u64) -> Result<Ext2Inode, FsError> {
    let (off, _inode_sz) = locate_inode(fs, ino)?;
    fs.read_struct(off).map_err(|_e| FsError::IOError)
}

fn locate_inode(fs: &mut Ext2DevHandle, ino: u64) -> Result<(u64, u64), FsError> {
    let (gno, idx) = get_ino_group_off(fs, ino);
    let inode_sz = fs.sb.inode_size as u64;
    let inode_tbl = get_block_group(fs, gno)?.inode_table_idx as u64;
    let offset = fs.block_size() * inode_tbl + idx * inode_sz;
    Ok((offset, inode_sz))
}
