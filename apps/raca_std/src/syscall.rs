#![allow(dead_code)]

use crate::{Result, result_from_isize};

#[naked]
extern "C" fn syscall_(
    _id: u64,
    _arg1: usize,
    _arg2: usize,
    _arg3: usize,
    _arg4: usize,
    _arg5: usize,
) -> isize {
    #[rustfmt::skip]
    unsafe {
        core::arch::naked_asm!(
            "mov r10,rcx",
            "syscall",
            "ret",
        )
    }
}

pub fn syscall(
    id: u64,
    arg1: usize,
    arg2: usize,
    arg3: usize,
    arg4: usize,
    arg5: usize,
) -> Result<usize> {
    let result = syscall_(id, arg1, arg2, arg3, arg4, arg5);

    result_from_isize(result)
}

#[naked]
pub extern "C" fn syscall_noret(
    _id: u64,
    _arg1: usize,
    _arg2: usize,
    _arg3: usize,
    _arg4: usize,
    _arg5: usize,
) -> ! {
    #[rustfmt::skip]
    unsafe {
        core::arch::naked_asm!(
            "mov r10,rcx",
            "syscall",
        )
    }
}
