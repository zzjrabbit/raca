#![no_std]
#![feature(naked_functions)]
#![feature(stmt_expr_attributes)]
#![feature(alloc_error_handler)]
#![feature(variant_count)]

extern crate alloc;

use core::panic::PanicInfo;

pub use task::exit;

pub mod debug;
mod error;
pub mod fs;
pub mod io;
pub mod kernel;
pub mod memory;
pub mod path;
mod syscall;
pub mod task;

pub use error::*;

extern "C" {
    fn main() -> usize;
}

pub fn dummy() {
    unsafe {
        core::arch::asm!("nop");
    }
}

#[no_mangle]
pub unsafe extern "sysv64" fn _start() -> ! {
    exit(main());
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("user panic: {}", info);
    exit(1);
}
