#![no_std]
#![feature(naked_functions)]
#![feature(stmt_expr_attributes)]

use core::panic::PanicInfo;

pub mod debug;
pub mod fs;
mod syscall;

extern "C" {
    fn main();
}

pub fn dummy() {
    unsafe {
        core::arch::asm!("nop");
    }
}

#[no_mangle]
pub unsafe extern "sysv64" fn _start() -> ! {
    main();
    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
