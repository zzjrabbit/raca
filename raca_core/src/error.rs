pub type Result<T> = ::core::result::Result<T, Error>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(isize)]
pub enum Error {
    InvalidUTF8String = -1,
    SignalNotFound = -2,
    InvalidOpenMode = -3,
    FileNotFound = -4,
    AccessDenied = -5,
    FileDescriptorNotFound = -6,
    InvalidSyscall = -7,
    ElfFileError = -8,
}
