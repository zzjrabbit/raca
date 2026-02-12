pub use crate::arch::mem::{
    KERNEL_ASPACE_BASE, KERNEL_ASPACE_SIZE, USER_ASPACE_BASE, USER_ASPACE_SIZE,
    current_page_table as kernel_page_table,
};
pub use mem::*;
pub use task::*;

mod mem;
mod task;

pub(crate) fn init() {
    mem::init();
}
