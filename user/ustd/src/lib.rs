#![no_std]
#![feature(rustc_private)]

use errors::Result;

extern crate alloc;

pub mod ipc;
pub mod os;
pub mod process;
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
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
