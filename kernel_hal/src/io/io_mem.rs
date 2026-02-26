use core::ops::Range;

use alloc::sync::Arc;
use errors::{Errno, Error};
use pod::Pod;

use crate::{
    mem::{
        PageProperty, PhysicalMemory, VirtAddr, VmSpace, align_down_by_page_size, phys_to_virt,
        virt_to_phys,
    },
    platform::mem::PAGE_SIZE,
};

#[derive(Debug)]
pub struct IoMem {
    start_address: VirtAddr,
    size: usize,
    vm_space: Arc<VmSpace>,
}

impl IoMem {
    pub fn acquire(range: Range<usize>) -> Result<Arc<Self>, Error> {
        let start = range.start;
        let size = range.end - range.start;

        let start_address = phys_to_virt(start);
        log::debug!("start address: {:x}", start_address);

        Ok(Arc::new(IoMem {
            start_address,
            size,
            vm_space: unsafe { VmSpace::kernel() },
        }))
    }
}

impl IoMem {
    pub fn start_address(&self) -> usize {
        self.start_address
    }

    pub fn size(&self) -> usize {
        self.size
    }
}

impl IoMem {
    fn try_map(&self) -> Result<(), Error> {
        let aligned_addr = align_down_by_page_size(self.start_address());
        unsafe { VmSpace::kernel() }
            .cursor(aligned_addr)?
            .map(
                &PhysicalMemory::from_start_address(
                    virt_to_phys(aligned_addr),
                    self.size().div_ceil(PAGE_SIZE),
                ),
                PageProperty::kernel_data(),
            )
            .unwrap();
        Ok(())
    }

    fn do_with_map(&self, mut f: impl FnMut() -> Result<(), Error>) -> Result<(), Error> {
        match f() {
            Ok(()) => Ok(()),
            Err(err) if err.errno() == Errno::PageFault => {
                self.try_map()?;
                f()
            }
            Err(err) => Err(err),
        }
    }

    pub fn read_bytes(&self, offset: usize, buffer: &mut [u8]) -> Result<(), Error> {
        let address = self.start_address() + offset;
        self.do_with_map(|| {
            self.vm_space
                .reader(address, buffer.len())
                .read_bytes(buffer)
        })
    }

    pub fn read<T: Pod>(&self, offset: usize, value: &mut T) -> Result<(), Error> {
        let address = self.start_address() + offset;
        self.do_with_map(|| self.vm_space.reader(address, size_of::<T>()).read(value))
    }

    pub fn write_bytes(&self, offset: usize, buffer: &[u8]) -> Result<(), Error> {
        let address = self.start_address() + offset;
        self.do_with_map(|| {
            self.vm_space
                .writer(address, buffer.len())
                .write_bytes(buffer)
        })
    }

    pub fn write<T: Pod>(&self, offset: usize, value: &T) -> Result<(), Error> {
        let address = self.start_address() + offset;
        self.do_with_map(|| self.vm_space.writer(address, size_of::<T>()).write(value))
    }
}
