pub mod task;
pub mod trap {
    #[repr(C)]
    #[derive(Debug, Default, Clone, Copy, Eq, PartialEq)]
    pub struct TrapFrame {}
}

pub fn init() {}

pub(crate) fn init_after_heap() {}
