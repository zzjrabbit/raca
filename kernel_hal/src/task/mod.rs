pub use crate::arch::task::UserContext;
pub use crate::platform::task::{HwThread, launch_multitask};
pub use exception::*;
pub use user::*;

mod exception;
mod user;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ThreadState {
    Running,
    #[default]
    Ready,
    Blocked,
}

impl ThreadState {
    pub fn blocked(&self) -> bool {
        matches!(self, Self::Blocked)
    }

    pub fn running(&self) -> bool {
        matches!(self, Self::Running)
    }

    pub fn ready(&self) -> bool {
        matches!(self, Self::Ready)
    }
}
