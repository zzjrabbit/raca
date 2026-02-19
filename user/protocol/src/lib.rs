#![no_std]

use pod::derive;

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod)]
pub struct ProcessStartInfo {
    pub process: u32,
    pub _reserved: u32,
    pub channel: u32,
    pub vmar: u32,
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

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod)]
pub struct WriteBuffer {
    pub addr: usize,
    pub len: usize,
}
