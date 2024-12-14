use syscall_macro::syscall;

use crate::Result;

pub fn debug(message: &str) -> Result<()> {
    const DEBUG_SYSCALL_ID: u64 = 0;
    syscall!(DEBUG_SYSCALL_ID, fn debug(message: *const u8, len: usize) -> Result<()>);

    debug(message.as_ptr(), message.len())
}
