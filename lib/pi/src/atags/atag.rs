use crate::atags::raw;

pub use crate::atags::raw::{Core, Mem};

/// An ATAG.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Atag {
    Core(raw::Core),
    Mem(raw::Mem),
    Cmd(&'static str),
    Unknown(u32),
    None,
}

impl Atag {
    /// Returns `Some` if this is a `Core` ATAG. Otherwise returns `None`.
    pub fn core(self) -> Option<Core> {
        match self {
            Atag::Core(c) => Some(c),
            _ => None,
        }
    }

    /// Returns `Some` if this is a `Mem` ATAG. Otherwise returns `None`.
    pub fn mem(self) -> Option<Mem> {
        match self {
            Atag::Mem(m) => Some(m),
            _ => None,
        }
    }

    /// Returns `Some` with the command line string if this is a `Cmd` ATAG.
    /// Otherwise returns `None`.
    pub fn cmd(self) -> Option<&'static str> {
        match self {
            Atag::Cmd(s) => Some(s),
            _ => None,
        }
    }
}

/// Safety: This function assumes the past u8 is a null terminated string
///         thus if it is not this function is UB
unsafe fn null_term_string(data: *const u8) -> &'static str {
    let mut len: usize = 0;
    let mut ptr = data;
    while *ptr != 0 {
        len += 1;
        ptr = ptr.offset(1);
    }

    let slice = core::slice::from_raw_parts(data, len);
    core::str::from_utf8(slice).unwrap()
}

impl From<&'static raw::Atag> for Atag {
    fn from(atag: &'static raw::Atag) -> Atag {
        // Safety: Tags are well defined, the tag determines what type of
        //          the union we need to cast too. Problem child is
        //          cmd which expects the cmd to be a null term string
        unsafe {
            match (atag.tag, &atag.kind) {
                (raw::Atag::CORE, &raw::Kind { core }) => Atag::Core(core),
                (raw::Atag::MEM, &raw::Kind { mem }) => Atag::Mem(mem),
                (raw::Atag::CMDLINE, &raw::Kind { ref cmd }) => {
                    Atag::Cmd(null_term_string(&cmd.cmd))
                }
                (raw::Atag::NONE, _) => Atag::None,
                (id, _) => Atag::Unknown(id),
            }
        }
    }
}
