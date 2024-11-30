use core::mem::{transmute, variant_count};

#[derive(Debug)]
#[allow(dead_code)]
#[repr(u8)]
pub enum SyscallIndex {
    Debug = 0,
    Open = 1,
    Malloc = 2,
    Read = 3,
    Write = 4,
    Lseek = 5,
    Close = 6,
    Fsize = 7,
    CreateProcess = 8,
    InsertModule = 9,
    HasSignal = 10,
    GetSignal = 11,
    DoneSignal = 12,
    StartWaitForSignal = 13,
    Exit = 14,
}

impl From<usize> for SyscallIndex {
    fn from(number: usize) -> Self {
        let syscall_length = variant_count::<Self>();
        if number >= syscall_length {
            panic!("Invalid syscall index: {}", number);
        }
        unsafe { transmute(number as u8) }
    }
}
