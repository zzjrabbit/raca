use core::mem::{transmute, variant_count};

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

impl From<isize> for Error {
    fn from(number: isize) -> Self {
        let error_length = variant_count::<Self>();
        if number >= error_length as isize {
            unreachable!()
        }
        unsafe { transmute(number) }
    }
}

pub(crate) fn result_from_isize<T>(res: isize) -> Result<T>
where
    T: From<usize>,
{
    if res < 0 {
        Err(Error::from(res))
    } else {
        Ok(T::from(res as usize))
    }
}
