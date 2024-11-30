use syscall_macro::syscall;

use crate::Result;

pub fn debug(message: &str) -> Result<()> {
    const DEBUG_SYSCALL_ID: u64 = 0;

    syscall!(DEBUG_SYSCALL_ID, message.as_ptr() as usize, message.len()).map(|_| ())
}
