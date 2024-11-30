#![no_std]
#![allow(improper_ctypes)]
#![allow(incomplete_features)]
#![feature(alloc_error_handler)]
#![feature(allocator_api)]
#![feature(abi_x86_interrupt)]
#![feature(naked_functions)]
#![feature(generic_const_exprs)]
#![feature(variant_count)]
#![feature(string_from_utf8_lossy_owned)]

extern crate alloc;

pub mod arch;
pub mod device;
pub mod error;
pub mod fs;
pub mod memory;
pub mod module;
pub mod syscall;
pub mod task;

pub fn init() {
    memory::init_heap();
    module::init_module();
    device::log::init();
    arch::smp::CPUS.write().init_bsp();
    arch::interrupts::IDT.load();
    arch::smp::CPUS.write().init_ap();
    arch::apic::init();
    syscall::init();
    fs::init();
    module::probe();
    task::init();

    log::info!("racaOS intialization completed!");
}
