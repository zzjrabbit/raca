use errors::Result;

use crate::{
    os::raca::{BorrowedHandle, OwnedHandle},
    syscall::sys_allocate_vmo,
};

pub struct Vmo {
    handle: OwnedHandle,
    count: usize,
}

impl Vmo {
    pub unsafe fn from_handle_count(handle: OwnedHandle, count: usize) -> Self {
        Vmo { handle, count }
    }

    pub fn allocate(count: usize) -> Result<Self> {
        let mut raw_handle = 0u32;
        unsafe {
            sys_allocate_vmo(count, &mut raw_handle)?;
            Ok(Vmo::from_handle_count(
                OwnedHandle::from_raw(raw_handle),
                count,
            ))
        }
    }
}

impl Vmo {
    pub fn count(&self) -> usize {
        self.count
    }

    pub(super) fn handle(&self) -> BorrowedHandle {
        self.handle.borrow()
    }
}
