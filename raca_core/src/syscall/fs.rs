use core::slice::from_raw_parts;

use alloc::string::String;

use crate::{error::RcResult, fs::{operation::{self, OpenMode}, Path}};

pub fn open(path_ptr: usize, path_len: usize, mode: usize) -> RcResult<usize> {
    let path_str = unsafe {
        from_raw_parts(path_ptr as *const u8, path_len)
    };
    let path = Path::new(String::from_utf8(path_str.to_vec()).unwrap());

    let mode = OpenMode::from_usize(mode)?;

    operation::open(path, mode)
}

