use crate::fs::*;

pub struct Ext2DirHandle {
    ino: u64,
    off: u64,
}

pub fn find_file(fs: &mut Ext2DevHandle, name: &[u8], cwd: u64) -> Result<Option<u64>, FsError> {
    const SLASH: u8 = 0x2F;
    let mut cursor = name;

    let mut cwd = if name[0] == SLASH {
        cursor = &cursor[1..];
        2
    } else {
        cwd
    };

    for sub in cursor.split(|&byte| byte == SLASH) {
        let name = String::from_utf8(sub.to_vec()).unwrap();
        println!("Looking for {name} in cwd_ino: {cwd}");
        let mut dir = open_dir(fs, cwd)?;
        'find_part: loop {
            match read_dir(fs, &mut dir)? {
                Some(dentry) => {
                    if &dentry.fname[0..] == sub {
                        cwd = dentry.ino;
                        println!("Found inode: {cwd}, itype: {}", dentry.file_type);
                        break 'find_part;
                    }
                }
                None => return Ok(None),
            }
        }
    }

    Ok(Some(cwd))
}

pub fn open_dir(fs: &mut Ext2DevHandle, ino: u64) -> Result<Ext2DirHandle, FsError> {
    let ino = if ino != 2 { 10 + ino } else { 2 };
    let inode = get_inode(fs, ino)?;

    println!("Inode: {inode:?}");

    if get_inode_type(&inode) == InodeType::Dir {
        Ok(Ext2DirHandle { ino, off: 0 })
    } else {
        Err(FsError::NotADir)
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
) -> Result<(Ext2DirEntry, Vec<u8>, u64), FsError> {
    if off >= inode.size as u64 {
        return Err(FsError::EndOfDir);
    }

    let byte_addr = get_inode_addr_from_offset(fs, inode, off)?;
    let dentry: Ext2DirEntry = fs.read_struct(byte_addr).map_err(|_| FsError::IOError)?;

    let mut data: Vec<u8> = Vec::with_capacity(dentry.name_len as usize);
    unsafe {
        data.set_len(dentry.name_len as usize);
    }
    let byte_addr = byte_addr + core::mem::size_of::<Ext2DirEntry>() as u64;
    fs.read_to_buf(byte_addr, &mut data[0..])
        .map_err(|_| FsError::IOError)?;

    Ok((dentry, data, dentry.rec_len as u64))
}

pub fn read_dir(
    fs: &mut Ext2DevHandle,
    dir: &mut Ext2DirHandle,
) -> Result<Option<DirLine>, FsError> {
    let inode = get_inode(fs, dir.ino)?;

    if dir.off >= inode.size as u64 {
        return Ok(None);
    }

    loop {
        let (entry, name, next_offset) = read_dentry(fs, &inode, dir.off)?;

        let fname = String::from_utf8(name.clone()).unwrap();
        println!("Looking for {fname} in inode: {entry:?}");

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
