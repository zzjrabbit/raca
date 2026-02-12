use alloc::sync::Arc;

use crate::task::ThreadState;

pub struct HwThread {}

impl HwThread {
    pub fn new() -> Self {
        Self {}
    }
}

impl HwThread {
    pub fn set_state(&self, _state: ThreadState) {}
    pub fn state(&self) -> ThreadState {
        ThreadState::Blocked
    }
    pub fn spawn(self: &Arc<Self>, _f: impl FnMut() + Send + 'static) {}
}
