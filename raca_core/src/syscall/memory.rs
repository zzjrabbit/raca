use x86_64::VirtAddr;

use crate::error::*;
use crate::memory::{ref_current_page_table, MappingType, MemoryManager};

pub fn malloc(address: usize, length: usize) -> Result<usize> {
    if length == 0 {
        return Ok(0);
    }

    MemoryManager::alloc_range(
        VirtAddr::new(address as u64),
        length as u64,
        MappingType::UserData.flags(),
        &mut unsafe { ref_current_page_table() },
    )
    .expect("Failed to allocate memory for malloc");

    Ok(length)
}
