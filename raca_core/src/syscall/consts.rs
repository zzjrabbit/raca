use core::mem::{transmute, variant_count};

#[derive(Debug)]
#[allow(dead_code)]
#[repr(u8)]
pub enum SyscallIndex {
    Debug,
    Open,
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