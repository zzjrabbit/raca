pub use crate::arch::{idle_ins, idle_loop};
pub(crate) use acpi::ACPI;
pub use logger::_print;

mod acpi;
mod logger;
pub(crate) mod mem;
mod panic;
pub(crate) mod task;

pub mod trap {
    pub use crate::arch::trap::{
        CpuExceptionInfo, disable_int, enable_int, inject_user_page_fault_handler,
    };
}

pub(crate) fn init() {
    mem::init();
    logger::init();
}
