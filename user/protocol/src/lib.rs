#![no_std]

use pod::derive;

pub const FIRST_HANDLE: u32 = 0;

pub const PROC_START_HANDLE_CNT: usize = 2;

pub const PROC_HANDLE_IDX: usize = 0;
pub const VMAR_HANDLE_IDX: usize = 1;

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod)]
pub struct ProcessStartInfo {
    pub vmar_base: usize,
    pub vmar_size: usize,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod)]
pub struct ReadBuffer {
    pub addr: usize,
    pub len: usize,
    pub actual_len_addr: usize,
}

impl ReadBuffer {
    pub fn new_zero() -> Self {
        Self {
            addr: 0,
            len: 0,
            actual_len_addr: 0,
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod)]
pub struct WriteBuffer {
    pub addr: usize,
    pub len: usize,
}
