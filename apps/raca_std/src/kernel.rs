use crate::Result;
use syscall_macro::syscall;

pub fn insert_module(data: &[u8]) -> Result<usize> {
    let ptr = data.as_ptr() as usize;
    let size = data.len();

    const INSERT_MODULE_SYSCALL_ID: u64 = 9;

    syscall!(INSERT_MODULE_SYSCALL_ID, fn insert_module(ptr: usize, size: usize) -> Result<usize>);
    
    insert_module(ptr, size)
}

const POWEROFF_SYSCALL_ID: u64 = 15;
syscall!(POWEROFF_SYSCALL_ID, pub fn poweroff());

const REBOOT_SYSCALL_ID: u64 = 16;
syscall!(REBOOT_SYSCALL_ID, pub fn reboot());
