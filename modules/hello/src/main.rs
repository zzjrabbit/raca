#![no_std]
#![no_main]
use module_std::{kernel_module, println, KernelModule};

kernel_module!(Hello, hello, Apache);

pub struct Hello;

impl KernelModule for Hello {
    fn init() -> Option<Self> {
        println!("hello init");
        Some(Self)
    }
}

impl Drop for Hello {
    fn drop(&mut self) {
        println!("hello drop");
    }
}
