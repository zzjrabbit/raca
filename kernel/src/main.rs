#![no_std]
#![no_main]

use limine::BaseRevision;

#[used]
#[unsafe(link_section = ".requests")]
static BASE_REVISION: BaseRevision = BaseRevision::with_revision(4);

#[unsafe(no_mangle)]
pub extern "C" fn kmain() -> ! {
    kernel::init();
    log::info!("KERNEL INITIALIZED!");
    kernel::arch::idle_loop();
}
