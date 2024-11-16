use alloc::alloc::Layout;
use talc::{ClaimOnOom, Span, Talc, Talck};
use x86_64::VirtAddr;

use super::MappingType;
use super::KERNEL_PAGE_TABLE;
use crate::memory::MemoryManager;

pub const HEAP_START: usize = 0x114514000000;
pub const HEAP_SIZE: usize = 128 * 1024 * 1024;
pub const HEAP_END: usize = HEAP_START + HEAP_SIZE;

#[global_allocator]
static ALLOCATOR: Talck<spin::Mutex<()>, ClaimOnOom> =
    Talc::new(unsafe { ClaimOnOom::new(Span::empty()) }).lock();

#[alloc_error_handler]
fn alloc_error_handler(layout: Layout) -> ! {
    panic!("Kernel heap allocation error: {:?}", layout)
}

pub fn init_heap() {
    let heap_start = VirtAddr::new(HEAP_START as u64);

    MemoryManager::alloc_range(
        heap_start,
        HEAP_SIZE as u64,
        MappingType::KernelData.flags(),
        &mut KERNEL_PAGE_TABLE.lock(),
    )
    .unwrap();

    unsafe {
        let arena = Span::from_base_size(heap_start.as_mut_ptr(), HEAP_SIZE);
        ALLOCATOR.lock().claim(arena).unwrap();
    }
}
