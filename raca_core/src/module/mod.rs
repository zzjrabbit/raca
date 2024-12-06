use core::{alloc::Layout, mem::transmute, ptr::NonNull};

use crate::memory::{ExtendedPageTable, HEAP_END, KERNEL_PAGE_TABLE, MappingType, MemoryManager};
use alloc::{
    collections::btree_map::BTreeMap,
    string::{String, ToString},
    sync::Arc,
    vec::Vec,
};
use goblin::elf::Elf;
use operations::*;
use talc::*;
use x86_64::VirtAddr;

mod load;
mod operations;
mod probe;

pub use probe::probe;

pub const MODULE_START: usize = HEAP_END;
pub const MODULE_SIZE: usize = 64 * 1024 * 1024;

static MODULE_ALLOCATOR: Talck<spin::Mutex<()>, ClaimOnOom> =
    Talc::new(unsafe { ClaimOnOom::new(Span::empty()) }).lock();

pub fn init_module() {
    let module_start = VirtAddr::new(MODULE_START as u64);

    MemoryManager::alloc_range(
        module_start,
        MODULE_SIZE as u64,
        MappingType::KernelCode.flags(),
        &mut KERNEL_PAGE_TABLE.lock(),
    )
    .unwrap();

    unsafe {
        let arena = Span::from_base_size(module_start.as_mut_ptr(), MODULE_SIZE);
        MODULE_ALLOCATOR.lock().claim(arena).unwrap();
    }
}

#[repr(C)]
pub struct InfoStruct {
    name: &'static str,
    license: &'static str,
}

pub struct Module {
    pub name: String,
    pub license: String,
    pub memory_sections: Vec<(u64, usize)>,
    symbol_addresses: BTreeMap<String, u64>,
    init_address: u64,
    exit_address: u64,
}

impl Drop for Module {
    fn drop(&mut self) {
        log::info!("{:x}", self.exit_address);
        let exit: extern "C" fn() -> usize = unsafe { transmute(self.exit_address) };
        let status = exit();

        for (start, size) in self.memory_sections.iter() {
            unsafe {
                MODULE_ALLOCATOR.lock().free(
                    NonNull::<u8>::new(*start as *mut u8).unwrap(),
                    Layout::from_size_align(*size, 4096).unwrap(),
                );
            }
        }

        log::info!("module {} unloaded( status {} )", self.get_name(), status);
    }
}
