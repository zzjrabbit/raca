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

    pub fn can_run(&self) -> bool {
        matches!(self, Self::Ready)
    }
}
