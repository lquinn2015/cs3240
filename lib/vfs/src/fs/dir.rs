
pub struct Ext2InodeHandle {
    ino: u64,
    off: u64,
}

impl Ext2InodeHandle {
    pub fn create(ino: u64) -> Ext2InodeHandle {
        Ext2InodeHandle { ino, off: 0 }
    }
