use limine::request::{HhdmRequest, MemoryMapRequest};
use spin::Lazy;

use crate::mem::{PageSize, PhysAddr, VirtAddr};
pub(crate) use frame::FRAME_ALLOCATOR;

mod frame;
mod heap;

pub const PAGE_SIZE: usize = PageSize::Size4K as usize;

#[used]
#[unsafe(link_section = ".requests")]
static HHDM_REQUEST: HhdmRequest = HhdmRequest::new();

#[used]
#[unsafe(link_section = ".requests")]
static MEMORY_MAP_REQUEST: MemoryMapRequest = MemoryMapRequest::new();

static PHYSICAL_MEMORY_OFFSET: Lazy<usize> =
    Lazy::new(|| HHDM_REQUEST.get_response().unwrap().offset() as usize);

pub fn phys_to_virt(physical_address: PhysAddr) -> VirtAddr {
    physical_address + *PHYSICAL_MEMORY_OFFSET
}

pub fn virt_to_phys(virtual_address: VirtAddr) -> PhysAddr {
    virtual_address - *PHYSICAL_MEMORY_OFFSET
}

pub(crate) fn init() {
    heap::init();
}
