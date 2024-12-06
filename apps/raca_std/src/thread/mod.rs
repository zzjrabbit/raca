use core::time::Duration;

use alloc::vec;
use syscall_macro::syscall;

use crate::Result;

const STACK_SIZE: usize = 8 * 1024;

pub fn spawn(f: fn()) -> Result<()> {
    let entry = f as usize;
    let stack = vec![0u8; STACK_SIZE];
    let stack_end = stack.leak().as_ptr() as usize + STACK_SIZE;

    const CREATE_THREAD_SYSCALL: u64 = 17;
    syscall!(CREATE_THREAD_SYSCALL, entry as usize, stack_end).map(|_| ())
}

pub fn yield_now() {
    const YIELD_SYSCALL: u64 = 18;
    let _ = syscall_macro::syscall!(YIELD_SYSCALL, 0, 0, 0, 0, 0);
}

pub fn sleep(duration: Duration) {
    const SLEEP_SYSCALL: u64 = 19;
    let _ = syscall_macro::syscall!(SLEEP_SYSCALL, duration.as_micros() as usize, 0, 0, 0, 0);
}
