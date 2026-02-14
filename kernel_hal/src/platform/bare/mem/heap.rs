use core::alloc::GlobalAlloc;

use good_memory_allocator::SpinLockedAllocator;

use crate::{
    arch::mem::current_page_table,
    mem::{
        CachePolicy, GeneralPageTable, MMUFlags, Page, PageProperty, PageSize,
        PhysicalMemoryAllocOptions, Privilege,
    },
};

#[global_allocator]
pub static ALLOCATOR: DefaultAllocator = DefaultAllocator::new();

pub struct DefaultAllocator(SpinLockedAllocator);

impl DefaultAllocator {
    pub const fn new() -> Self {
        DefaultAllocator(SpinLockedAllocator::empty())
    }
}

impl Default for DefaultAllocator {
    fn default() -> Self {
        Self::new()
    }
}

unsafe impl GlobalAlloc for DefaultAllocator {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        unsafe { self.0.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
        unsafe {
            self.0.dealloc(ptr, layout);
        }
    }
}

impl DefaultAllocator {
    #[allow(static_mut_refs)]
    fn init(&self) {
        const HEAP_SIZE: usize = 4 * 1024 * 1024;
        const HEAP_START: usize = 0xffffc00000000000;

        let page_size = PageSize::Size4K;
        let page_count = HEAP_SIZE / page_size as usize;

        let pm = PhysicalMemoryAllocOptions::new()
            .count(page_count)
            .allocate()
            .unwrap();
        let property = PageProperty::new(
            MMUFlags::READ | MMUFlags::WRITE,
            CachePolicy::CacheCoherent,
            Privilege::KernelOnly,
        );

        let mut page_table = current_page_table();

        for id in 0..page_count {
            page_table
                .map(
                    Page::new_aligned(HEAP_START + id * page_size as usize, page_size),
                    pm.get_start_address_of_frame(id).unwrap(),
                    property,
                )
                .unwrap();
        }

        unsafe {
            self.0.init(HEAP_START, HEAP_SIZE);
        }
    }
}

pub fn init() {
    ALLOCATOR.init();
}
