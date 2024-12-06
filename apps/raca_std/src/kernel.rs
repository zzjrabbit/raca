use crate::Result;
use syscall_macro::syscall;

pub fn insert_module(data: &[u8]) -> Result<usize> {
    let ptr = data.as_ptr() as usize;
    let size = data.len();

    const INSERT_MODULE_SYSCALL_ID: u64 = 9;

    syscall!(INSERT_MODULE_SYSCALL_ID, ptr, size)
}

pub fn poweroff() {
    const POWEROFF_SYSCALL_ID: u64 = 15;

    let _ = syscall!(POWEROFF_SYSCALL_ID);
}

pub fn reboot() {
    const REBOOT_SYSCALL_ID: u64 = 16;

    let _ = syscall!(REBOOT_SYSCALL_ID);
}
