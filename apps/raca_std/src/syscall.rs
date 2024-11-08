#![allow(dead_code)]


#[naked]
pub extern "C" fn syscall(_id: u64, _arg1: usize, _arg2: usize, _arg3: usize, _arg4: usize, _arg5: usize) -> usize {
    #[rustfmt::skip]
    unsafe {
        core::arch::naked_asm!(
            "syscall",
            "ret",
        )
    }
}

#[naked]
pub extern "C" fn syscall_noret(_id: u64, _arg1: usize, _arg2: usize, _arg3: usize, _arg4: usize, _arg5: usize) -> ! {
    #[rustfmt::skip]
    unsafe {
        core::arch::naked_asm!(
            "syscall",
        )
    }
}