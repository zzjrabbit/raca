use core::sync::atomic::{AtomicU64, Ordering};

use alloc::sync::Arc;
use kernel_hal::{platform::TaskContext, task::ThreadState};

use crate::{impl_kobj, new_kobj, object::KObjectBase};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ThreadId(u64);

impl Default for ThreadId {
    fn default() -> Self {
        Self::new()
    }
}

impl ThreadId {
    pub fn new() -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);
        Self(NEXT_ID.fetch_add(1, Ordering::SeqCst))
    }
}

pub struct Thread {
    tid: ThreadId,
    base: KObjectBase,
    ctx: Arc<TaskContext>,
}

impl_kobj!(Thread);

impl Thread {
    pub fn new() -> Arc<Self> {
        new_kobj!({
            tid: ThreadId::new(),
            ctx: Arc::new(TaskContext::new()),
        })
    }
}

impl Thread {
    pub fn id(&self) -> ThreadId {
        self.tid
    }

    pub fn state(&self) -> ThreadState {
        self.ctx.state()
    }

    pub fn set_state(&self, state: ThreadState) {
        self.ctx.set_state(state);
    }

    pub fn context(&self) -> Arc<TaskContext> {
        self.ctx.clone()
    }
}

impl Thread {
    pub fn start(self: &Arc<Self>, update_fn: impl FnMut() + Send + 'static) {
        self.set_state(ThreadState::Ready);
        self.context().spawn(update_fn);
    }
}

#[cfg(test)]
mod tests {
    use alloc::vec;
    use kernel_hal::arch::task::UserContext;

    use super::*;
    use core::time::Duration;
    use std::prelude::rust_2024::*;

    extern crate std;

    #[test]
    fn new_thread() {
        let thread = Thread::new();
        assert_eq!(thread.state(), ThreadState::Ready);
    }

    #[test]
    fn start_thread() {
        let thread = Thread::new();
        thread.start(|| {
            std::println!("Thread started");
        });
        std::thread::sleep(Duration::from_millis(100));
        thread.set_state(ThreadState::Blocked);
    }

    #[test]
    fn user_thread() {
        fn entry_point() {
            loop {}
        }

        let stack = Vec::leak(vec![0u8; 8 * 1024]);

        let mut user_ctx = UserContext::default();
        user_ctx.set_ip(entry_point as *const () as usize);
        user_ctx.set_sp(stack.as_ptr() as usize + stack.len());

        let thread = Thread::new();
        thread.start(move || {
            user_ctx.enter_user_space();
        });
        std::thread::sleep(Duration::from_millis(100));
        thread.set_state(ThreadState::Blocked);
    }
}
