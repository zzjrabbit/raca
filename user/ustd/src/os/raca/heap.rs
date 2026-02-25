use core::alloc::GlobalAlloc;

use spin::Mutex;
use talc::{ClaimOnOom, Span, Talc, Talck};

use crate::vm::{MMUFlags, Vmar, Vmo};

#[global_allocator]
static HEAP: Heap = Heap::new();

pub fn init(root_vmar: &Vmar) {
    HEAP.init(root_vmar);
}

struct Heap {
    inner: Talck<Mutex<()>, ClaimOnOom>,
}

const HEAP_SIZE: usize = 16 * 1024 * 1024 * 1024;

impl Heap {
    const fn new() -> Self {
        Self {
            inner: Talck::new(Talc::new(unsafe { ClaimOnOom::new(Span::empty()) })),
        }
    }

    fn init(&self, root_vmar: &Vmar) {
        let vmar = root_vmar.allocate(HEAP_SIZE).unwrap();
        let vmo = Vmo::allocate(vmar.page_count()).unwrap();
        vmar.map(0, &vmo, MMUFlags::READ | MMUFlags::WRITE).unwrap();
        //crate::println!("allocate vmar: {:#x} {:#x}", vmar.base(), vmar.size());
        unsafe {
            self.inner
                .lock()
                .claim(Span::from_base_size(vmar.base() as *mut u8, vmar.size()))
                .unwrap();
        }
    }
}

unsafe impl GlobalAlloc for Heap {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        unsafe { self.inner.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
        unsafe { self.inner.dealloc(ptr, layout) }
    }

    unsafe fn realloc(
        &self,
        ptr: *mut u8,
        layout: core::alloc::Layout,
        new_size: usize,
    ) -> *mut u8 {
        unsafe { self.inner.realloc(ptr, layout, new_size) }
    }
}
