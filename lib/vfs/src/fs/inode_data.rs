use crate::fs::*;

//let byte_addr = get_inode_addr_from_offset(fs, inode, off)?;
pub fn get_inode_addr_from_offset(
    fs: &mut Ext2DevHandle,
    inode: &Ext2Inode,
    off: u64,
) -> Result<u64, FsError> {
    let block_sz = fs.block_size() as u64;
    let inode_block = off / block_sz;
    let block_off = off % block_sz;

    let real_block = match get_inode_block(fs, inode, inode_block)? {
        Some(blk) => blk,
        _ => return Err(FsError::BlockOutOfBounds),
    };
    let byte_addr = real_block * block_sz + block_off;

    Ok(byte_addr)
}

#[derive(Debug)]
enum BlockPos {
    Direct(u64),
    Indirect1(u64),
    Indirect2(u64, u64),
    Indirect3(u64, u64, u64),
    OutOfRange,
}

fn inode_block_to_pos(fs: &mut Ext2DevHandle, inode_block: u64) -> BlockPos {
    let indirect_1_sz = fs.block_size() / 4;
    let indirect_2_sz = indirect_1_sz * indirect_1_sz;
    let indirect_3_sz = indirect_2_sz * indirect_1_sz;

    if inode_block < 12 {
        BlockPos::Direct(inode_block)
    } else if inode_block < 12 + indirect_1_sz {
        BlockPos::Indirect1(inode_block - 12)
    } else if inode_block < 12 + indirect_1_sz + indirect_2_sz {
        let base = inode_block - 12 - indirect_1_sz;
        BlockPos::Indirect2(base / indirect_1_sz, base % indirect_1_sz)
    } else if inode_block < 12 + indirect_1_sz + indirect_2_sz + indirect_3_sz {
        let base = inode_block - 12 - indirect_1_sz - indirect_2_sz;
        BlockPos::Indirect3(
            base / indirect_2_sz,                   // which double indirect block
            (base % indirect_2_sz) / indirect_1_sz, // inside that double block which indirect1
            (base % indirect_2_sz) % indirect_1_sz, // inside that indirect1 which direct
        )
    } else {
        BlockPos::OutOfRange
    }
}

fn read_indirect(fs: &mut Ext2DevHandle, indirect: u64, index: u64) -> Result<u64, FsError> {
    let byte_addr = indirect * fs.block_size() + index * 4;
    let iblk: u32 = fs.read_struct(byte_addr).map_err(|_e| FsError::IOError)?;

    Ok(iblk as u64)
}

/// Given a Inode block (offset) translate it to a real_block (absolute)
///     solve all indirection
#[rustfmt::skip]
fn get_inode_block(
    fs: &mut Ext2DevHandle,
    inode: &Ext2Inode,
    inode_block: u64,
) -> Result<Option<u64>, FsError> {
    use BlockPos::*;

    match inode_block_to_pos(fs, inode_block) {
        Direct(l0) => Ok(Some(inode.blocks[l0 as usize] as u64)),
        Indirect1(l0) => {
            let block1 = inode.blocks[12];
            if block1 == 0 { return Ok(None); }
            let block0 = read_indirect(fs, block1 as u64, l0)?;
            if block0 == 0 { return Ok(None); } else { return Ok(Some(block0)); }
        }
        Indirect2(l1, l0) => {
            let block2 = inode.blocks[13];
            if block2 == 0 { return Ok(None); }
            let block1 = read_indirect(fs, block2 as u64, l1)?;
            if block1 == 0 { return Ok(None); }
            let block0 = read_indirect(fs, block1, l0)?;
            if block0 == 0 { return Ok(None); } else { return Ok(Some(block0)); }
        }
        Indirect3(l2, l1, l0) => {
            let block3 = inode.blocks[14];
            if block3 == 0 { return Ok(None); }
            let block2 = read_indirect(fs, block3 as u64, l2)?;
            if block2 == 0 { return Ok(None); }
            let block1 = read_indirect(fs, block2, l1)?;
            if block1 == 0 { return Ok(None); }
            let block0 = read_indirect(fs, block1, l0)?; 
            if block0 == 0 { return Ok(None); } else { return Ok(Some(block0)); }
        }
        BlockPos::OutOfRange => Err(FsError::BlockOutOfBounds),
    }
}
