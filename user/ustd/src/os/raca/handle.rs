use crate::syscall::sys_remove_handle;

pub type RawHandle = u32;

#[derive(Debug)]
pub struct OwnedHandle(RawHandle);

impl OwnedHandle {
    pub const unsafe fn from_raw(raw: RawHandle) -> Self {
        Self(raw)
    }

    pub const unsafe fn as_raw(&self) -> RawHandle {
        self.0
    }
}

impl OwnedHandle {
    pub fn borrow(&self) -> BorrowedHandle {
        BorrowedHandle(self.0)
    }
}

impl Drop for OwnedHandle {
    fn drop(&mut self) {
        unsafe {
            sys_remove_handle(self.as_raw()).unwrap();
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct BorrowedHandle(RawHandle);

impl BorrowedHandle {
    pub const unsafe fn from_raw(raw: RawHandle) -> Self {
        Self(raw)
    }

    pub const unsafe fn as_raw(&self) -> RawHandle {
        self.0
    }
}
