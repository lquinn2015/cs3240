mod fs;
mod traits;

use fs::*;

fn main() {
    let mut handle = Ext2DevHandle::mount("unix.img".to_owned()).unwrap();
    println!("{:?}", handle.get_superblock());

    let gd = read_block_group(&mut handle, 0);
    println!("Block group 0:  \n {:?}", gd);

    println!("sizeof inode is: {}", core::mem::size_of::<fs::Ext2Inode>());

    println!("Inode root: \n {:x?}", read_inode(&mut handle, 2));

    let mut dir = open_dir(&mut handle, 2).unwrap();
    println!("Reading Dir: ");
    while let Ok(Some(dline)) = read_dir(&mut handle, &mut dir) {
        if let Ok(name) = core::str::from_utf8(&dline.fname) {
            println!("io: {}, ty: {}, {name}", dline.ino, dline.file_type);
        }
    }

    let mut dir = open_dir(&mut handle, 12).unwrap();
    println!("Reading Dir: ");
    while let Ok(Some(dline)) = read_dir(&mut handle, &mut dir) {
        if let Ok(name) = core::str::from_utf8(&dline.fname) {
            println!("io: {}, ty: {}, {name}", dline.ino, dline.file_type);
        }
    }

    print_inode_alloc_tbl(&mut handle, 0);

    let tmp_ino = find_file(&mut handle, &"/tmp2/blamp/mor.txt".as_bytes(), 2)
        .unwrap()
        .unwrap();
    println!("inode for more.txt: {:?}", get_inode(&mut handle, tmp_ino));

    //for i in 0..20 {
    //    let tmp = get_inode(&mut handle, tmp_ino + i);
    //    println!("ino: {}: {tmp:?}", tmp_ino + i);
    //}

    // Iterator<Dirs>
    //handle.read_dir("/");

    // file handle to a.txt
    //let fd = handle.open("/a.txt");
    //let mut buf = [u8; 32];
    //fd.read(buf);
    //println!("{buf:?}");
}
