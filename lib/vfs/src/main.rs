mod fs;
mod traits;

use fs::*;

fn main() {
    let mut handle = Ext2DevHandle::mount("ext2_image.img".to_owned()).unwrap();
    println!("{:?}", handle.get_superblock());

    let gd = read_block_group(&mut handle, 0);
    println!("Block group 0:  \n {:?}", gd);

    println!("sizeof inode is: {}", core::mem::size_of::<fs::Ext2Inode>());

    println!("Inode root: \n {:x?}", read_inode(&mut handle, 2));

    let mut fp = Ext2InodeHandle::create(2);

    //let mut fp = Ext2InodeHandle::create(handle, 2);
    let mut buf = [0; 512];

    fp.read(&mut handle, &mut buf[0..]);

    println!("{:?}", buf);

    // Iterator<Dirs>
    //handle.read_dir("/");

    // file handle to a.txt
    //let fd = handle.open("/a.txt");
    //let mut buf = [u8; 32];
    //fd.read(buf);
    //println!("{buf:?}");
}
