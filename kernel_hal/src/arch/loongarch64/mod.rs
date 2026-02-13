use loongarch64::registers::init_pwc;

pub mod mem;
pub mod task;
mod trap;

pub(crate) fn init() {
    init_pwc();
    trap::init();
}
