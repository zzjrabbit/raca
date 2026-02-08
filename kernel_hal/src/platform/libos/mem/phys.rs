use spin::{Lazy, Mutex};

use crate::{
    mem::{BitmapFrameAllocator, PageSize},
    platform::mem::PMEM_SIZE,
};

pub static FRAME_ALLOCATOR: Lazy<Mutex<BitmapFrameAllocator>> = Lazy::new(|| {
    let usable_frames = PMEM_SIZE / PageSize::Size4K as usize;
    let bitmap_buffer = vec![0; PMEM_SIZE / usize::BITS as usize].leak();

    let mut allocator = BitmapFrameAllocator::new_all_free(bitmap_buffer);
    allocator.deallocate_frames(0, usable_frames);

    Mutex::new(allocator)
});
