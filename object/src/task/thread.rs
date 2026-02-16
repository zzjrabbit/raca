use core::sync::atomic::{AtomicU64, Ordering};

use alloc::sync::{Arc, Weak};
use kernel_hal::{
    mem::PageProperty,
    task::{HwThread, ThreadState},
};

use crate::{
    impl_kobj,
    mem::{Vmar, Vmo},
    object::KObjectBase,
    task::Process,
};

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
    process: Weak<Process>,
    tid: ThreadId,
    base: KObjectBase,
    ctx: Arc<HwThread>,
}

impl_kobj!(Thread);

impl Thread {
    pub fn new(process: Weak<Process>) -> Arc<Self> {
        static KERNEL_STACK_SIZE: usize = 32 * 1024;

        Arc::new_cyclic(|this: &Weak<Self>| Self {
            process,
            tid: ThreadId::new(),
            base: KObjectBase::default(),
            ctx: Arc::new(HwThread::new(this.clone(), || {
                let vmar = Vmar::kernel();
                let stack = vmar.allocate_child(KERNEL_STACK_SIZE).unwrap();
                let vmo = Vmo::allocate_ram(stack.page_count()).unwrap();
                stack
                    .direct_map(0, &vmo, PageProperty::kernel_data())
                    .unwrap();
                stack.end()
            })),
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

    pub fn context(&self) -> Arc<HwThread> {
        self.ctx.clone()
    }

    pub fn process(&self) -> Option<Arc<Process>> {
        self.process.upgrade()
    }
}

impl Thread {
    pub fn current() -> Option<Arc<Self>> {
        HwThread::current_thread()
            .upgrade()
            .and_then(|thread| thread.downcast().ok())
    }
}

impl Thread {
    pub fn start(self: &Arc<Self>, update_fn: impl FnMut() + Send + 'static) {
        self.context().spawn(update_fn);
    }
}

#[cfg(test)]
mod tests {
    use alloc::vec;
    use kernel_hal::task::UserContext;

    use super::*;
    use core::time::Duration;
    use std::prelude::rust_2024::*;

    extern crate std;

    #[test]
    fn new_thread() {
        let thread = Thread::new(Weak::new());
        assert_eq!(thread.state(), ThreadState::Ready);
    }

    #[test]
    fn start_thread() {
        let thread = Thread::new(Weak::new());
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

        let thread = Thread::new(Weak::new());
        thread.start(move || {
            user_ctx.enter_user_space();
        });
        std::thread::sleep(Duration::from_millis(100));
        thread.set_state(ThreadState::Blocked);
    }
}
