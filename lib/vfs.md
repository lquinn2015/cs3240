
VFS - Virtual File System
    the purpose is include the Traits for a a FS 

```rs
trait FileSystem {
        type File: File;
        type Dir: Dir<Entry = Self::Entry>;
        type Entry: Entry<Self::Dir, Self::File>; 

        fn open<P: AsRef<Path>>(self, path: P) -> io::Result<Entry>;
        fn open_file<P: AsRef<Path>>(self, path: P) -> io::Result<File>;  
            // derivable
        fn open_dir<P: AsRef<Path>>(self, path: P) -> io::Result<Dir>;
            // derivable

        fn remove<P: AsRef<Path>>(self, path: P) -> io::Result<Entry>;
        fn remove_file<P: AsRef<Path>>(self, path: P) -> io::Result<File>;  
            // derivable
        fn remove_dir<P: AsRef<Path>>(self, path: P) -> io::Result<Dir>;
            // derivable

}

trait File: io::write + io::read + io::seek + sized {
    fn sync() -> io::Result<()>;
    fn size() -> u64;
};

trait Dir: Size {
    type Entry Entry;
    type Iter: Iterator<Item = Self::Entry>;
    fn entries(&self) -> io::Result<Self::Iter>;
};




```

