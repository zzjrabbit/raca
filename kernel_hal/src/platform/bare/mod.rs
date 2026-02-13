pub use crate::arch::mem::{
    KERNEL_ASPACE_BASE, KERNEL_ASPACE_SIZE, USER_ASPACE_BASE, USER_ASPACE_SIZE,
    current_page_table as kernel_page_table,
};
pub(crate) use acpi::ACPI;
pub use logger::_print;
pub use mem::*;
pub use task::*;

mod acpi;
mod logger;
mod mem;
mod panic;
mod task;

pub(crate) fn init() {
    mem::init();
    logger::init();
}
