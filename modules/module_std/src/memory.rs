#![allow(improper_ctypes)]

use core::alloc::{GlobalAlloc, Layout};

extern "C" {
    fn alloc(layout: Layout) -> *mut u8;
    fn dealloc(ptr: *mut u8, layout: Layout);
}

#[global_allocator]
static ALLOCATOR: MyAllocator = MyAllocator;

struct MyAllocator;

unsafe impl GlobalAlloc for MyAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        dealloc(ptr, layout)
    }
}
