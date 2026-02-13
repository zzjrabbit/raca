use core::ops::Range;

use alloc::sync::Arc;
use errors::Error;

use crate::mem::{Pod, VirtAddr, VmSpace, phys_to_virt};

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
        log::info!("start address: {:x}", start_address);

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
    pub fn read_bytes(&self, offset: usize, buffer: &mut [u8]) -> Result<(), Error> {
        let address = self.start_address() + offset;
        self.vm_space
            .reader(address, buffer.len())
            .read_bytes(buffer)
    }

    pub fn read<T: Pod>(&self, offset: usize, value: &mut T) -> Result<(), Error> {
        let address = self.start_address() + offset;
        self.vm_space.reader(address, size_of::<T>()).read(value)
    }

    pub fn write_bytes(&self, offset: usize, buffer: &[u8]) -> Result<(), Error> {
        let address = self.start_address() + offset;
        self.vm_space
            .writer(address, buffer.len())
            .write_bytes(buffer)
    }

    pub fn write<T: Pod>(&self, offset: usize, value: &T) -> Result<(), Error> {
        let address = self.start_address() + offset;
        self.vm_space.writer(address, size_of::<T>()).write(value)
    }
}
