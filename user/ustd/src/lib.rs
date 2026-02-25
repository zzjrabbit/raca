#![no_std]
#![feature(rustc_private)]
#![feature(int_roundings)]

use errors::Result;

pub use stdio::_print;

extern crate alloc;

pub mod ipc;
pub mod os;
pub mod process;
mod stdio;
pub mod syscall;
pub mod thread;
pub mod vm;

pub fn debug(msg: &str) -> Result<()> {
    unsafe {
        syscall::sys_debug(msg.as_ptr(), msg.len())?;
    }
    Ok(())
}

pub fn dummy() {}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    crate::println!("USER PANIC: {}", info);
    loop {}
}
