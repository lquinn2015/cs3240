mod fs;
mod traits;

use fs::Ext2DevHandle;
use fs::Ext2Inode;

fn main() {
    let mut handle = Ext2DevHandle::mount("ext2_image.img".to_owned()).unwrap();
    println!("{:?}", handle.read_superblock());

    let gd = handle.read_block_group(0);
    println!("Block group 0:  \n {:?}", gd);

    println!("sizeof inode is: {}", core::mem::size_of::<fs::Ext2Inode>());

    let root = handle.read_inode(2);
    println!("Inode 11: \n {:x?}", root);

    // Iterator<Dirs>
    //handle.read_dir("/");

    // file handle to a.txt
    //let fd = handle.open("/a.txt");
    //let mut buf = [u8; 32];
    //fd.read(buf);
    //println!("{buf:?}");
}
