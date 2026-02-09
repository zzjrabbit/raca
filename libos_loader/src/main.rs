use std::time::Duration;

use kernel_hal::{
    mem::{PageProperty, VirtAddr},
    task::ThreadState,
};
use object::{
    mem::Vmo,
    task::{Process, Thread},
};

extern "C" fn entry_point(_vdso_ptr: usize) {
    #[allow(clippy::empty_loop)]
    loop {}
}

fn main() {
    env_logger::init();
    kernel_hal::platform::init();

    const STACK_SIZE: usize = 8 * 1024 * 1024;

    let process = Process::new();

    let stack = process.root_vmar().allocate_child(STACK_SIZE).unwrap();
    stack
        .map(
            0,
            &Vmo::allocate_ram(stack.page_count()).unwrap(),
            PageProperty::user_data(),
            false,
        )
        .unwrap();

    let thread = Thread::new();
    process.start(
        thread.clone(),
        entry_point as *const () as VirtAddr,
        stack.end(),
    );
    std::thread::sleep(Duration::from_millis(100));
    thread.set_state(ThreadState::Blocked);
}
