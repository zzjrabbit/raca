#![no_std]

extern crate alloc;

use core::fmt::{Debug, Display};

use alloc::string::String;

pub type Result<T> = core::result::Result<T, Error>;

impl Errno {
    pub fn with_message<S: Into<String>>(&self, message: S) -> Error {
        Error {
            errno: *self,
            message: message.into(),
        }
    }

    pub fn no_message(&self) -> Error {
        Error {
            errno: *self,
            message: String::new(),
        }
    }
}

pub struct Error {
    errno: Errno,
    message: String,
}

impl Error {
    pub fn errno(&self) -> Errno {
        self.errno
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

impl From<Error> for i32 {
    fn from(error: Error) -> Self {
        error.errno as i32
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}: {}", self.errno, self.message)
    }
}

impl Debug for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}: {}", self.errno, self.message)
    }
}

impl core::error::Error for Error {}

#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Errno {
    NotFound = 1,
    AccessDenied = 2,
    BadHandle = 3,
    InvArg = 4,
    WrongType = 5,
    NotSupported = 6,
    PeerClosed = 7,
    ShouldWait = 8,
    OutOfMemory = 9,
    NotMapped = 10,
    PageFault = 11,
    TooBig = 12,
    MapFailed = 13,
    InvSyscall = 14,
}
