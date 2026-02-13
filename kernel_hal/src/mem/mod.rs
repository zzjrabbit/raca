#[cfg(not(feature = "libos"))]
pub use crate::arch::mem::{
    KERNEL_ASPACE_BASE, KERNEL_ASPACE_SIZE, USER_ASPACE_BASE, USER_ASPACE_SIZE,
    current_page_table as kernel_page_table,
};
use crate::platform::mem::PAGE_SIZE;
#[cfg(feature = "libos")]
pub use crate::platform::mem::{
    KERNEL_ASPACE_BASE, KERNEL_ASPACE_SIZE, USER_ASPACE_BASE, USER_ASPACE_SIZE, kernel_page_table,
};
pub use crate::platform::mem::{phys_to_virt, virt_to_phys};
pub use frame::*;
pub use page_table::*;
pub use physical::*;
pub use vm_space::*;

mod frame;
mod page_table;
mod physical;
mod vm_space;

pub const fn align_down_by_page_size(addr: usize) -> usize {
    addr / PAGE_SIZE * PAGE_SIZE
}

pub const fn align_up_by_page_size(addr: usize) -> usize {
    align_down_by_page_size(addr + PAGE_SIZE - 1)
}
