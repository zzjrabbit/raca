use std::time::Duration;

use kernel_hal::{arch::task::UserContext, task::ThreadState};
use object::task::Thread;
use syscalls::Sysno;

fn entry_point() {
    loop {
        let msg = "[A]";
        unsafe {
            syscalls::raw_syscall!(Sysno::write, 0, msg.as_ptr() as usize, msg.len());
        }
    }
}

fn main() {
    env_logger::init();
    kernel_hal::platform::init();

    const STACK_SIZE: usize = 1 * 1024 * 1024;

    let stack = Vec::leak(vec![0u8; STACK_SIZE]);

    let mut user_ctx = UserContext::default();
    user_ctx.set_ip(entry_point as *const () as usize);
    user_ctx.set_sp(stack.as_mut_ptr() as usize + STACK_SIZE);

    let thread = Thread::new();
    thread.start(move || {
        user_ctx.enter_user_space();
    });
    std::thread::sleep(Duration::from_secs(1));
    thread.set_state(ThreadState::Blocked);
}
