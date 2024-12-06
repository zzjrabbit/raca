use core::slice::{from_raw_parts, from_raw_parts_mut};

use alloc::string::String;

use crate::fs::{
    Path,
    operation::{self, OpenMode},
};

use crate::error::*;

pub fn open(path_ptr: usize, path_len: usize, mode: usize) -> Result<usize> {
    let path_str = unsafe { from_raw_parts(path_ptr as *const u8, path_len) };
    let path = Path::new(String::from_utf8(path_str.to_vec()).unwrap());

    let mode = OpenMode::from_usize(mode)?;

    operation::open(path, mode)
}

pub fn read(fd: usize, buf_ptr: usize, buf_len: usize) -> Result<usize> {
    let buf = unsafe { from_raw_parts_mut(buf_ptr as *mut u8, buf_len) };

    operation::read(fd, buf)
}

pub fn write(fd: usize, buf_ptr: usize, buf_len: usize) -> Result<usize> {
    let buf = unsafe { from_raw_parts(buf_ptr as *const u8, buf_len) };

    operation::write(fd, buf)
}

pub fn lseek(fd: usize, offset: usize) -> Result<usize> {
    operation::lseek(fd, offset)
}

pub fn close(fd: usize) -> Result<usize> {
    operation::close(fd)
}

pub fn fsize(fd: usize) -> Result<usize> {
    operation::fsize(fd)
}
