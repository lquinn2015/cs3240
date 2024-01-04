use core::fmt;
use core::option::Option::{self, None, Some};
use core::result;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug)]
pub struct Error {
    pub kind: ErrorKind,
    pub msg: &'static str,
}

#[macro_export]
macro_rules! const_io_error {
    ($kind:expr, $msg:expr $(,)?) => {
        Error {
            kind: $kind,
            msg: $msg,
        }
    };
}

impl Error {
    pub fn new(kind: ErrorKind, msg: &'static str) -> Self {
        Error { kind, msg }
    }

    pub fn is_interrupted(&self) -> bool {
        match self.kind {
            ErrorKind::Interrupted => true,
            _ => false,
        }
    }
    pub fn kind(&self) -> ErrorKind {
        self.kind.clone()
    }
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!("{:?}, {}", self.kind, self.msg))
    }
}

impl core::error::Error for Error {}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErrorKind {
    NotFound,
    PermissionDenied,
    ConnectionRefused,
    ConnectionReset,
    HostUnreachable,
    NetworkUnreachable,
    ConnectionAborted,
    NotConnected,
    AddrInUse,
    AddrNotAvailable,
    NetworkDown,
    BrokenPipe,
    AlreadyExists,
    WouldBlock,
    NotADirectory,
    IsADirectory,
    DirectoryNotEmpty,
    ReadOnlyFilesystem,
    FilesystemLoop,
    StaleNetworkFileHandle,
    InvalidInput,
    InvalidData,
    TimedOut,
    WriteZero,
    StorageFull,
    NotSeekable,
    FilesystemQuotaExceeded,
    FileTooLarge,
    ResourceBusy,
    ExecutableFileBusy,
    Deadlock,
    CrossesDevices,
    TooManyLinks,
    InvalidFilename,
    ArgumentListTooLong,
    Interrupted,
    Unsupported,
    UnexpectedEof,
    OutOfMemory,
    Other,
}

#[derive(Debug)]
pub struct Error {
    pub kind: ErrorKind,
    pub msg: &'static str,
}

#[macro_export]
macro_rules! const_io_error {
    ($kind:expr, $msg:expr $(,)?) => {
        Error {
            kind: $kind,
            msg: $msg,
        }
    };
}

impl Error {
    pub fn new(kind: ErrorKind, msg: &'static str) -> Self {
        Error { kind, msg }
    }

    pub fn is_interrupted(&self) -> bool {
        match self.kind {
            ErrorKind::Interrupted => true,
            _ => false,
        }
    }
    pub fn kind(&self) -> ErrorKind {
        self.kind.clone()
    }
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!("{:?}, {}", self.kind, self.msg))
    }
}

impl core::error::Error for Error {}

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErrorKind {
    NotFound,
    PermissionDenied,
    ConnectionRefused,
    ConnectionReset,
    HostUnreachable,
    NetworkUnreachable,
    ConnectionAborted,
    NotConnected,
    AddrInUse,
    AddrNotAvailable,
    NetworkDown,
    BrokenPipe,
    AlreadyExists,
    WouldBlock,
    NotADirectory,
    IsADirectory,
    DirectoryNotEmpty,
    ReadOnlyFilesystem,
    FilesystemLoop,
    StaleNetworkFileHandle,
    InvalidInput,
    InvalidData,
    TimedOut,
    WriteZero,
    StorageFull,
    NotSeekable,
    FilesystemQuotaExceeded,
    FileTooLarge,
    ResourceBusy,
    ExecutableFileBusy,
    Deadlock,
    CrossesDevices,
    TooManyLinks,
    InvalidFilename,
    ArgumentListTooLong,
    Interrupted,
    Unsupported,
    UnexpectedEof,
    OutOfMemory,
    Other,
}
