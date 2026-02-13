#![no_std]
#![no_main]

use limine::BaseRevision;

#[used]
#[unsafe(link_section = ".requests")]
static BASE_REVISION: BaseRevision = BaseRevision::with_revision(4);

#[unsafe(no_mangle)]
pub extern "C" fn kmain() -> ! {
    kernel_hal::init();
    log::info!("kernel initialized");
    kernel_hal::arch::enable_int();
    kernel_hal::arch::idle_loop();
}
