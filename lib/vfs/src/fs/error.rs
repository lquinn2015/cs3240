#[derive(Debug)]
pub enum FsError {
    IOError,
    BlockOutOfBounds,
    NotADir,
    EndOfDir,
}

impl std::fmt::Display for FsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FsError::IOError => f.write_str("IOError"),
            FsError::BlockOutOfBounds => f.write_str("Block Out of Range"),
            FsError::NotADir => f.write_str("open_dir on non dir"),
            FsError::EndOfDir => f.write_str("EndOfDir"),
        }
    }
}

impl core::error::Error for FsError {}
