#![no_std]
#![feature(rustc_private)]

use protocol::ProcStartInfo;

pub mod syscall;

pub fn debug(msg: &str) {
    unsafe {
        syscall::sys_debug(msg.as_ptr(), msg.len());
    }
}

unsafe extern "C" {
    fn main() -> i32;
}

#[unsafe(no_mangle)]
extern "C" fn _start(_info: *const ProcStartInfo) -> ! {
    unsafe {
        main();
    }

    loop {}
}

pub fn dummy() {}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
