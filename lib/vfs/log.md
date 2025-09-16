

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

#






# sources i liked
[1] https://www.science.smith.edu/~nhowe/262/oldlabs/ext2.html

