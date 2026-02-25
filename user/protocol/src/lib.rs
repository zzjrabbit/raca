#![no_std]

use pod::derive;

pub const FIRST_HANDLE: u32 = 0;

pub const PROC_START_HANDLE_CNT: usize = 2;

pub const PROC_HANDLE_IDX: usize = 0;
pub const VMAR_HANDLE_IDX: usize = 1;

pub const BOOT_HANDLE_CNT: usize = 2;

pub const BOOT_TERM_HANDLE_IDX: usize = 0;
pub const BOOT_FB_HANDLE_IDX: usize = 1;

pub const BOOT_DATA_CNT: usize = 3;

pub const TERM_SIZE_IDX: usize = 0;
pub const FB_WIDTH_IDX: usize = 1;
pub const FB_HEIGHT_IDX: usize = 2;

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
