use errors::Result;

use crate::{
    os::raca::OwnedHandle,
    syscall::{
        sys_allocate_vmar, sys_allocate_vmar_at, sys_map_vmar, sys_protect_vmar, sys_unmap_vmar,
    },
    vm::{MMUFlags, PAGE_SIZE, Vmo},
};

pub struct Vmar {
    handle: OwnedHandle,
    base: usize,
    size: usize,
}

impl Vmar {
    pub unsafe fn from_handle_base_size(handle: OwnedHandle, base: usize, size: usize) -> Self {
        Self { handle, base, size }
    }
}

impl Vmar {
    pub fn allocate(&self, size: usize) -> Result<Self> {
        let mut raw_handle = 0u32;
        let base = unsafe { sys_allocate_vmar(self.handle.as_raw(), size, &mut raw_handle)? };
        Ok(Self {
            handle: unsafe { OwnedHandle::from_raw(raw_handle) },
            base,
            size,
        })
    }

    pub fn allocate_at(&self, base: usize, size: usize) -> Result<Self> {
        let mut raw_handle = 0u32;
        let base =
            unsafe { sys_allocate_vmar_at(self.handle.as_raw(), base, size, &mut raw_handle)? };
        Ok(Self {
            handle: unsafe { OwnedHandle::from_raw(raw_handle) },
            base,
            size,
        })
    }
}

impl Vmar {
    pub fn base(&self) -> usize {
        self.base
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn end(&self) -> usize {
        self.base + self.size
    }

    pub fn page_count(&self) -> usize {
        self.size / PAGE_SIZE
    }
}

impl Vmar {
    pub fn map(&self, offset: usize, vmo: &Vmo, flags: MMUFlags) -> Result<()> {
        unsafe {
            sys_map_vmar(
                self.handle.as_raw(),
                offset,
                vmo.handle().as_raw(),
                flags.bits(),
            )?;
        }
        Ok(())
    }

    pub fn unmap(&self, addr: usize, size: usize) -> Result<()> {
        unsafe {
            sys_unmap_vmar(self.handle.as_raw(), addr, size)?;
        }
        Ok(())
    }

    pub fn protect(&self, addr: usize, size: usize, flags: MMUFlags) -> Result<()> {
        unsafe {
            sys_protect_vmar(self.handle.as_raw(), addr, size, flags.bits())?;
        }
        Ok(())
    }
}
