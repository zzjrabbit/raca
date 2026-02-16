use core::alloc::GlobalAlloc;

use spin::Mutex;
use talc::{ClaimOnOom, Span, Talc, Talck};

use crate::{
    arch::mem::current_page_table,
    mem::{
        CachePolicy, GeneralPageTable, MMUFlags, Page, PageProperty, PageSize,
        PhysicalMemoryAllocOptions, Privilege,
    },
};

#[global_allocator]
pub static ALLOCATOR: DefaultAllocator = DefaultAllocator::new();

pub struct DefaultAllocator(Talck<Mutex<()>, ClaimOnOom>);

impl DefaultAllocator {
    pub const fn new() -> Self {
        DefaultAllocator(Talck::new(Talc::new(unsafe {
            ClaimOnOom::new(Span::empty())
        })))
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

const HEAP_SIZE: usize = 16 * 1024 * 1024;
const HEAP_START: usize = 0xffff_c000_0000_0000;

impl DefaultAllocator {
    fn init(&self) {
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
            self.0
                .lock()
                .claim(Span::from_base_size(HEAP_START as _, HEAP_SIZE))
                .unwrap();
        }
    }
}

pub fn init() {
    ALLOCATOR.init();
}
