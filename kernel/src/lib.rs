#![no_std]
#![feature(sync_unsafe_cell)]

extern crate alloc;

pub use error::*;

use crate::arch::enable_int;

mod acpi;
pub mod arch;
mod error;
pub mod framebuffer;
pub mod io;
pub mod irq;
pub mod logger;
pub mod mem;
pub mod obj;
mod panic;
pub mod sync;
pub mod task;
pub mod timer;

pub fn init() {
    mem::init();
    arch::init();
    logger::init();

    #[cfg(feature = "smp")]
    arch::init_smp();

    enable_int();
}
