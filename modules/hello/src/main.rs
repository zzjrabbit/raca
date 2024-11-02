#![no_std]
#![no_main]
use module_std::{kernel_module, println};

kernel_module!(hello, GPL, init, exit);

pub fn init() -> usize {
    0
}

pub fn exit() -> usize {
    println!("Kernel module hello exiting");
    0
}
