use crate::{
    Errno, Error,
    mem::{FRAME_ALLOCATOR, PageSize, PhysicalAddress, convert_physical_to_virtual},
};

pub struct PhysicalMemoryAllocOptions {
    count: usize,
}

impl PhysicalMemoryAllocOptions {
    pub const fn new() -> Self {
        Self { count: 1 }
    }
}

impl Default for PhysicalMemoryAllocOptions {
    fn default() -> Self {
        Self::new()
    }
}

impl PhysicalMemoryAllocOptions {
    /// Set the number of frames to allocate.
    pub fn count(mut self, count: usize) -> Self {
        self.count = count;
        self
    }
}

impl PhysicalMemoryAllocOptions {
    /// Allocate physical memory frames with the specified options.
    pub fn allocate(self) -> Result<PhysicalMemory, Error> {
        PhysicalMemory::new(self.count)
    }
}

#[derive(Debug)]
pub struct PhysicalMemory {
    count: usize,
    start_address: PhysicalAddress,
}

impl PhysicalMemory {
    fn new(count: usize) -> Result<Self, Error> {
        let start_address = FRAME_ALLOCATOR
            .lock()
            .allocate_frames(count)
            .ok_or(Errno::ENOMEM.no_message())?;

        Ok(Self {
            count,
            start_address,
        })
    }
}

impl PhysicalMemory {
    pub fn from_start_address(start_address: PhysicalAddress, count: usize) -> Self {
        Self {
            count,
            start_address,
        }
    }

    pub fn containing_address(address: PhysicalAddress, count: usize) -> Self {
        let start_address = PageSize::Size4K.align_down(address);

        Self::from_start_address(start_address, count)
    }

    pub fn deallocate(&self) {
        for id in 0..self.count() {
            let start_address = self.get_start_address_of_frame(id).unwrap();
            FRAME_ALLOCATOR.lock().deallocate_frames(start_address, 1);
        }
    }
}

impl PhysicalMemory {
    pub fn as_slice(&self, id: usize) -> Result<&[u8], Error> {
        let paddr = self.get_start_address_of_frame(id)?;
        let vaddr = convert_physical_to_virtual(paddr);

        Ok(unsafe { core::slice::from_raw_parts(vaddr as *const u8, PageSize::Size4K as usize) })
    }

    pub fn as_mut_slice(&mut self, id: usize) -> Result<&mut [u8], Error> {
        let paddr = self.get_start_address_of_frame(id)?;
        let vaddr = convert_physical_to_virtual(paddr);

        Ok(unsafe { core::slice::from_raw_parts_mut(vaddr as *mut u8, PageSize::Size4K as usize) })
    }
}

impl PhysicalMemory {
    pub fn get_start_address_of_frame(&self, id: usize) -> Result<PhysicalAddress, Error> {
        if id >= self.count() {
            return Err(Errno::EINVAL.no_message());
        }

        Ok(self.start_address + (id * PageSize::Size4K as usize))
    }

    pub fn count(&self) -> usize {
        self.count
    }
}

impl PhysicalMemory {
    pub fn read_bytes(&self, offset: usize, buffer: &mut [u8]) -> Result<(), Error> {
        let page_size = PageSize::Size4K;

        if (self.count() * page_size as usize) < (offset + buffer.len()) {
            return Err(Errno::EINVAL.no_message());
        }

        let virtual_address = convert_physical_to_virtual(self.start_address + offset);

        unsafe {
            core::ptr::copy_nonoverlapping(
                virtual_address as *const u8,
                buffer.as_mut_ptr(),
                buffer.len(),
            );
        }

        Ok(())
    }

    pub fn write_bytes(&self, offset: usize, buffer: &[u8]) -> Result<(), Error> {
        let page_size = PageSize::Size4K;

        if (self.count() * page_size as usize) < (offset + buffer.len()) {
            return Err(Errno::EINVAL.no_message());
        }

        let virtual_address = convert_physical_to_virtual(self.start_address + offset);

        unsafe {
            core::ptr::copy_nonoverlapping(
                buffer.as_ptr(),
                virtual_address as *mut u8,
                buffer.len(),
            );
        }

        Ok(())
    }

    pub fn fill(&self, byte: u8) -> Result<(), Error> {
        let page_size = PageSize::Size4K;

        let virtual_address = convert_physical_to_virtual(self.start_address);

        unsafe {
            core::ptr::write_bytes(virtual_address as *mut u8, byte, page_size as usize);
        }

        Ok(())
    }

    pub fn zero(&self) -> Result<(), Error> {
        self.fill(0)
    }
}
