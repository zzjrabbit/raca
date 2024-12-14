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
    
    syscall!(CREATE_THREAD_SYSCALL, fn create_thread(entry: usize, stack_end: usize) -> Result<()>);
    
    create_thread(entry, stack_end)
}

const YIELD_SYSCALL: u64 = 18;
syscall!(YIELD_SYSCALL, pub fn yield_now());

pub fn sleep(duration: Duration) -> Result<()> {
    const SLEEP_SYSCALL: u64 = 19;
    syscall!(SLEEP_SYSCALL, fn sleep(duration: usize) -> Result<()>);
    
    sleep(duration.as_millis() as usize)
}

