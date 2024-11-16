#![no_std]
#![feature(naked_functions)]
#![feature(stmt_expr_attributes)]
#![feature(alloc_error_handler)]

extern crate alloc;

use core::panic::PanicInfo;

pub mod debug;
pub mod fs;
pub mod memory;
pub mod path;
pub mod task;
pub mod io;
pub mod kernel;
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
    memory::init_heap();
    main();
    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("user panic: {}",info);
    loop {}
}
