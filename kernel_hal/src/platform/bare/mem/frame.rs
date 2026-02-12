use limine::memory_map::EntryType;
use spin::{Lazy, Mutex};

use super::phys_to_virt;
use crate::{
    mem::{BitmapFrameAllocator, PhysAddr},
    platform::mem::MEMORY_MAP_REQUEST,
};

pub(crate) static FRAME_ALLOCATOR: Lazy<Mutex<BitmapFrameAllocator>> = Lazy::new(|| {
    let memory_map = MEMORY_MAP_REQUEST.get_response().unwrap();
    let memory_size = memory_map
        .entries()
        .last()
        .map(|region| region.base + region.length)
        .expect("No memory regions found");

    let bitmap_size = (memory_size / 4096).div_ceil(8) as usize;

    let usable_regions = memory_map
        .entries()
        .iter()
        .filter(|region| region.entry_type == EntryType::USABLE);

    let bitmap_address = usable_regions
        .clone()
        .find(|region| region.length >= bitmap_size as u64)
        .map(|region| region.base)
        .expect("No suitable memory region for bitmap");

    let bitmap_buffer = unsafe {
        let physical_address = bitmap_address as PhysAddr;
        let virtual_address = phys_to_virt(physical_address);
        let bitmap_inner_size = bitmap_size / size_of::<usize>();
        core::slice::from_raw_parts_mut(virtual_address as *mut usize, bitmap_inner_size)
    };

    let mut origin_frames = 0;

    for region in usable_regions.clone() {
        let frame_count = (region.length / 4096) as usize;

        origin_frames += frame_count;
    }

    let bitmap_frame_count = bitmap_size.div_ceil(4096);

    let usable_frames = origin_frames - bitmap_frame_count;

    let mut allocator = BitmapFrameAllocator::new(bitmap_buffer, usable_frames);

    for region in usable_regions {
        let start_page_index = (region.base / 4096) as usize;
        let frame_count = (region.length / 4096) as usize;

        allocator.deallocate_frames(start_page_index, frame_count);
    }

    Mutex::new(allocator)
});
