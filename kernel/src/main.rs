#![no_std]
#![no_main]

use kernel_hal::task::launch_multitask;
use limine::BaseRevision;
use object::task::Thread;

#[used]
#[unsafe(link_section = ".requests")]
static BASE_REVISION: BaseRevision = BaseRevision::with_revision(4);

#[unsafe(no_mangle)]
pub extern "C" fn kmain() -> ! {
    kernel_hal::init();
    log::info!("kernel initialized");
    let thread_a = Thread::new();
    thread_a.start(|| {
        kernel_hal::print!("[A]");
    });
    let thread_b = Thread::new();
    thread_b.start(|| {
        kernel_hal::print!("[B]");
    });
    launch_multitask();

    kernel_hal::platform::idle_loop();
}
