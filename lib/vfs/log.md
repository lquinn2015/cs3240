

# Background 

Learning Rust is fun and I wanted to challenge myself by building a Rust
inspired Operating system. I've been following a Course called cs3210 but I
found the courses FAT filesystem structure not very good. The goal below is to
track my work to implement a Ext2 filesystem with a VFS OS abstraction. I've
made neither but I noticed everyone says its a great way to learn but there is a
i was struggling to put it together together so I thought I'd log it. It
might be easier to get something link Minix's FS working but Ext2 is a real OS
that has been deployed and one day if I got this working I might learn how
Journaling works in a File System which is a huge stretch goal.


# File System Stack

I want to build a stack for a File system like this 

    [UserSpace]
    [Syscall API]
    [VFS] - API traits - resolves which Filesystem a file opens.
      |     Has File / Dir concepts exposed to user  
      |
      |- Ext2 impl
      |
      \- Minix 2 impl

    [Block device] - This is the how the FS interacts with the block device 
                    this can be a simple File back to a linux file supported 
                    the parent OS or in the Future VirtIO in Qemu 
    [Disk] - Physical Media could be just an linux file or actual hardware

You can see its my opinion that the File System lives at least a top some block
device which suppose basic read/write sector. And the FS layer organizes how
that physical medium is managed. 


# Ext2 
 
   Block Group                                                                      
 xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx                                      
 x boot blk  x  super blk  x Block Group x  block   x inode   x  block x inode x                                     
 x  1024 b   x             x descriptors x  bit map x bit map x  table x table x                                      
 x           x   1 block   x    N blocks x   1Block x  1Block x  N blk x N blk x                                      
 xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx                                      
                                                                                
Ext2 is a boot block followed by multiple block groups. I am going to just
support 1 block group. In the Block group is really the super block and beyond.
Super block contains the sizing information of the FS #inodes size of blocks in
the file System. Ext2 supports sector sizes of 1K 2K 4K etc. The big the the
block the worst Fragementation can be but also the max number of blocks, and
inodes you can have. Lets consider we have a 1K sector size in the Filesystem
this means the block/inode bit map support 1024 * 8 blocks and inodes

# Inode / Block Bitmaps 
    
This maps are local to the Block Group they exist in. 
        MSB                 LSB  
byte 0: b7 b6 b5 b4 b3 b2 b1 b0  -------- b0 points to block 1
byte 1:
byte 2: 

# Redesign
Defs: Basic defs for inode, sb, block group, dir(-name)
Filesystem - driver to read/write buffers
    Data 
        Cache: 
            Superblock  - 1Kbytes
            BlockGroups - #num blocks * 256b
            [Inode; 32] - slots  32 * 256b  # use HashMap swap to LRU  
            

Inode
    R/O stage
        locate_inode
        read_inode(fs, ino) -> Result<Inode,()>
            Read Inode from FS block device
        get_inode(fs, ino) -> Result<Inode, ()> 
            gets an inode from the cache 
            if not in cache read from block dev 
        locate_inode(fs, ino) -> Result<(Offset'byte, sz)>
            locates where this inode is
    R/W stage
        write_inode
        update_inode
Group 
    read_group() 
    read_group_desc() -> Result<GroupDesrp ()>
    get_ino_group -> Result<(u64,u64),()>
        (group number, ino in group)
    get_block_group() -> Result<(u64,u64), ()>
        (group number, block in group)



# Reading a Directory

This should be the first thing you implement! Why? This will require you to
build several functions that are used to inplement default file behavior but
also it allows you an entry point into the file system. You know the numerical
root / ino. 

Ideal you following this path you can slowly implement the following calls 

open_dir(fs, ino) -> Result<dir handle>  this is your special file pointer it also
checks that the ino is a directory type. 

read_dir(fs, dir) -> Result<Option(Dentry, Line)> 
This function will return an entry and a line which is bytes for the file name
it also updates the offset. Subsequent calls advance the directory pointer to
the next dir. This is easy to turn into an iterator. To do this you need a few

read_dentry(fs, dir) -> Result<(dentry, name, next_offset)> - this is a good
wrapper and allows you to simplify the read_dir impl to a loop until there is no
more valid dentry or 

read_





# sources i liked
[1] https://www.science.smith.edu/~nhowe/262/oldlabs/ext2.html
[2] https://github.com/honzasp/libext2  - good impl
