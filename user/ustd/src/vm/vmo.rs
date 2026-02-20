use errors::Result;
use pod::Pod;

use crate::{
    os::raca::{BorrowedHandle, OwnedHandle},
    syscall::{sys_allocate_vmo, sys_read_vmo, sys_write_vmo},
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

impl Vmo {
    pub fn read(&self, offset: usize, buffer: &mut [u8]) -> Result<()> {
        unsafe {
            sys_read_vmo(
                self.handle.as_raw(),
                offset,
                buffer.as_mut_ptr(),
                buffer.len(),
            )?;
        }
        Ok(())
    }

    pub fn read_val<T: Pod>(&self, offset: usize) -> Result<T> {
        let mut value = T::new_zeroed();
        self.read(offset, value.as_mut_bytes())?;
        Ok(value)
    }

    pub fn write(&self, offset: usize, buffer: &[u8]) -> Result<()> {
        unsafe {
            sys_write_vmo(self.handle.as_raw(), offset, buffer.as_ptr(), buffer.len())?;
        }
        Ok(())
    }

    pub fn write_val<T: Pod>(&self, offset: usize, value: &T) -> Result<()> {
        let bytes = value.as_bytes();
        self.write(offset, bytes)
    }
}
