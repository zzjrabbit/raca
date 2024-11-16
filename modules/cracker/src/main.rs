#![no_std]
#![no_main]
use module_std::{kernel_module, println, KernelModule};

kernel_module!(Cracker, cracker, Apache);

pub struct Cracker;

impl KernelModule for Cracker {
    fn init() -> Option<Self> {
        println!("\x1b[31m Haha! I'm going to crack your kernel! Haha! Crack!\x1b[0m");
        unsafe {
            core::arch::asm!("cli;hlt");
        }
        Some(Self)
    }
}

impl Drop for Cracker {
    fn drop(&mut self) {
    }
}
