use crate::os::raca::{BorrowedHandle, OwnedHandle};

pub struct Thread {
    handle: OwnedHandle,
}

impl Thread {
    pub unsafe fn from_handle(handle: OwnedHandle) -> Self {
        Self { handle }
    }
}

impl Thread {
    #[allow(unused)]
    pub(crate) fn handle(&self) -> BorrowedHandle {
        self.handle.borrow()
    }
}
