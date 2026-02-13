use loongarch64::registers::init_pwc;

pub use trap::{disable_int, enable_int};

pub mod mem;
pub(crate) mod serial;
pub mod task;
mod trap;

pub(crate) fn init() {
    init_pwc();
    trap::init();
}

pub(crate) fn init_after_heap() {
    serial::init();
}

pub fn idle_ins() {
    #[cfg(not(feature = "libos"))]
    unsafe {
        core::arch::asm!("idle 0");
    }
}

pub fn idle_loop() -> ! {
    loop {
        idle_ins();
    }
}
