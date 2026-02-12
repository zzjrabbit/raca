pub mod mem;
pub mod task;
mod trap;

pub(crate) fn init() {
    trap::init();
}
