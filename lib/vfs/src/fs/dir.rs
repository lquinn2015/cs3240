use crate::fs::*;

pub struct Ext2DirHandle {
    ino: u64,
    off: u64,
}

pub fn open_dir(fs: &mut Ext2DevHandle, ino: u64) -> Result<Ext2DirHandle, ()> {
    let inode = get_inode(fs, ino)?;

    if get_inode_type(&inode) == InodeType::Dir {
        Ok(Ext2DirHandle { ino, off: 0 })
    } else {
        Err(())
    }
}

#[derive(Debug)]
pub struct DirLine {
    pub ino: u64,
    pub file_type: u8,
    pub fname: Vec<u8>,
}

pub fn read_dentry(
    fs: &mut Ext2DevHandle,
    inode: &Ext2Inode,
    off: u64,
) -> Result<(Ext2DirEntry, Vec<u8>, u64), ()> {
    let byte_addr = get_inode_addr_from_offset(fs, inode, off)?;
    let dentry: Ext2DirEntry = fs.read_struct(byte_addr).map_err(|_| ())?;

    let mut data: Vec<u8> = Vec::with_capacity(dentry.name_len as usize);
    unsafe {
        data.set_len(dentry.name_len as usize);
    }
    let byte_addr = byte_addr + core::mem::size_of::<Ext2DirEntry>() as u64;
    fs.read_to_buf(byte_addr, &mut data[0..]).map_err(|_| ())?;

    Ok((dentry, data, dentry.rec_len as u64))
}

pub fn read_dir(fs: &mut Ext2DevHandle, dir: &mut Ext2DirHandle) -> Result<Option<DirLine>, ()> {
    let inode = get_inode(fs, dir.ino)?;

    if dir.off >= inode.size as u64 {
        return Ok(None);
    }

    loop {
        let (entry, name, next_offset) = read_dentry(fs, &inode, dir.off)?;

        dir.off += next_offset;
        if entry.ino != 0 {
            return Ok(Some(DirLine {
                ino: entry.ino as u64,
                file_type: entry.file_type,
                fname: name,
            }));
        }
    }
}
