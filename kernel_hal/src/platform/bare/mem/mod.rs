use limine::request::{HhdmRequest, MemoryMapRequest};
use loongarch64::registers::init_pwc;
use spin::Lazy;

pub(crate) use frame::FRAME_ALLOCATOR;

use crate::mem::PageSize;

mod frame;

pub type PhysAddr = usize;
pub type VirtAddr = usize;

pub const PAGE_SIZE: usize = PageSize::Size4K as usize;

pub const fn align_down_by_page_size(addr: usize) -> usize {
    addr / PAGE_SIZE * PAGE_SIZE
}

pub const fn align_up_by_page_size(addr: usize) -> usize {
    align_down_by_page_size(addr + PAGE_SIZE - 1)
}

pub(super) fn init() {
    init_pwc();
}

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
