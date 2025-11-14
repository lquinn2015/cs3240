use crate::fs::error::FsError;

use super::defs::*;
use super::fs::*;

// Stub for cached block group
pub fn get_block_group(fs: &mut Ext2DevHandle, gno: u64) -> Result<Ext2GroupDesc, FsError> {
    Ok(read_block_group(fs, gno)?)
}

pub fn read_block_group(fs: &mut Ext2DevHandle, gno: u64) -> Result<Ext2GroupDesc, FsError> {
    let block_size = fs.block_size() as usize;
    let bgs = fs.sb.block_count / fs.sb.blocks_per_group;
    assert!(gno <= bgs as u64);

    let desc_off = gno as u64 * core::mem::size_of::<Ext2GroupDesc>() as u64;

    let bg_offset = if (1024 + core::mem::size_of::<Ext2SuperBlock>()) < block_size {
        block_size as u64
    } else {
        todo!("sb + 1024 > block")
    };

    let byte_addr: u64 = bg_offset + desc_off;

    fs.read_struct(byte_addr).map_err(|_e| FsError::IOError)
}

pub fn print_inode_alloc_tbl(fs: &mut Ext2DevHandle, gno: u64) {
    let bg = get_block_group(fs, gno).unwrap();
    println!("{bg:?}");
}

fn get_ino_group_off(fs: &Ext2DevHandle, ino: u64) -> (u64, u64) {
    let group_sz = fs.sb.inodes_per_group as u64;
    ((ino - 1) / group_sz, (ino - 1) % group_sz)
}

fn get_blocks_group_off(fs: &Ext2DevHandle, bno: u64) -> (u64, u64) {
    let group_sz = fs.sb.blocks_per_group as u64;
    let rel_block = bno - fs.sb.first_data_block as u64;
    (rel_block / group_sz, rel_block % group_sz)
}
