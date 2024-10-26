#![no_std]
#![no_main]
use module_std::{get_module_handle, kernel_module, println};

kernel_module!(hello, GPL, init, exit);

pub fn init() -> usize {
    println!("hello handle: {}", get_module_handle());
    0
}

pub fn exit() -> usize {
    println!("Kernel module hello exiting");
    0
}
