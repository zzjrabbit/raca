pub(crate) mod mem;
pub mod power;
pub mod task;

pub use crate::arch::trap;

pub fn init() {}

pub fn idle_loop() {
    std::process::exit(0);
}
