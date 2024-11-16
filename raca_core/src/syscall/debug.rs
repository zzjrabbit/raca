use core::str;

use crate::error::{RcError, RcResult};

pub fn debug(ptr: usize, len: usize) -> RcResult<usize> {
    let data = unsafe { core::slice::from_raw_parts(ptr as *const u8, len) };
    let msg = str::from_utf8(data).map_err(|_| RcError::INVALID_ARGS)?;

    crate::print!(
        "{}",
        msg,
    );

    crate::serial_print!(
        "{}",
        msg,
    );

    

    Ok(0)
}
