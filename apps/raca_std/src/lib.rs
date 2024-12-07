#![no_std]
#![feature(naked_functions)]
#![feature(stmt_expr_attributes)]
#![feature(alloc_error_handler)]
#![feature(variant_count)]
#![feature(exact_size_is_empty)]
#![allow(unsafe_op_in_unsafe_fn)]

extern crate alloc;

use core::panic::PanicInfo;

pub use task::exit;

pub mod debug;
pub mod env;
mod error;
pub mod fs;
pub mod io;
pub mod kernel;
pub mod memory;
pub mod path;
pub mod process;
mod syscall;
mod task;
pub mod thread;

pub use error::*;

unsafe extern "C" {
    fn main() -> usize;
}

pub fn dummy() {
    unsafe {
        core::arch::asm!("nop");
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "sysv64" fn _start(
    argv: usize,
    argc: usize,
    env_start: usize,
    env_len: usize,
) -> ! {
    env::ENV_INFO.lock().env_start = env_start;
    env::ENV_INFO.lock().env_len = env_len;
    env::ARG_INFO.lock().argc = argc;
    env::ARG_INFO.lock().argv = argv;
    exit(main());
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("user panic: {}", info);
    exit(1);
}
