use core::cell::{Cell, SyncUnsafeCell};

use alloc::{boxed::Box, sync::Arc, vec::Vec};
use spin::Mutex;

use crate::{
    arch::task::{TaskContext, context_switch, first_context_switch, kernel_task_entry_wrapper},
    platform::task::sched::SCHEDULER,
    task::ThreadState,
};

mod sched;

#[unsafe(no_mangle)]
pub(crate) extern "C" fn kernel_task_entry() -> ! {
    let mut updater = {
        let current_thread = SCHEDULER.lock().current().unwrap();
        current_thread.func.get().unwrap()
    };

    loop {
        updater();
        schedule();
    }
}

pub struct HwThread {
    inner: Mutex<HwThreadInner>,
    ctx: SyncUnsafeCell<TaskContext>,
    func: FuncWrapper,
}

struct HwThreadInner {
    state: ThreadState,
}

impl Default for HwThread {
    fn default() -> Self {
        Self::new()
    }
}

impl HwThread {
    pub fn new() -> Self {
        let mut ctx = TaskContext::new();
        ctx.set_ip(kernel_task_entry_wrapper as *const () as usize);
        ctx.set_sp(Vec::leak(alloc::vec![0u8; 8 * 1024]).as_ptr() as usize);
        Self {
            inner: Mutex::new(HwThreadInner {
                state: ThreadState::Blocked,
            }),
            ctx: SyncUnsafeCell::new(ctx),
            func: FuncWrapper::new(),
        }
    }
}

impl HwThread {
    /// Set state.
    /// It adds and removes the thread from the scheduler atomatically.
    pub fn set_state(self: &Arc<Self>, state: ThreadState) {
        let origin = self.state();
        self.inner.lock().state = state;
        if origin.ready() && !state.ready() {
            SCHEDULER.lock().remove(self);
        }
        if !origin.ready() && state.ready() {
            SCHEDULER.lock().add(self);
        }
    }
    pub fn state(&self) -> ThreadState {
        self.inner.lock().state
    }

    pub fn spawn(self: &Arc<Self>, f: impl FnMut() + Send + 'static) {
        self.func.set(Box::new(f));
        self.set_state(ThreadState::Ready);
    }
}

pub fn launch_multitask() {
    let next = SCHEDULER.lock().get_next();
    // Access directly to avoid unnecessary checks.
    next.inner.lock().state = ThreadState::Running;
    let next_ctx = next.ctx.get();

    unsafe {
        first_context_switch(next_ctx);
    }
}

#[inline(always)]
pub(super) fn schedule() {
    let current = SCHEDULER.lock().current().unwrap();
    let current_ctx = current.ctx.get();

    let next_ctx = {
        let next = SCHEDULER.lock().get_next();
        // Access directly to avoid unnecessary checks.
        next.inner.lock().state = ThreadState::Running;
        next.ctx.get()
    };

    if current.state().running() {
        current.inner.lock().state = ThreadState::Ready;
        SCHEDULER.lock().add(&current);
    }

    unsafe {
        context_switch(next_ctx, current_ctx);
    }
}

type Entry = Box<dyn FnMut() + Send + 'static>;

struct FuncWrapper(Cell<Option<Entry>>);

unsafe impl Send for FuncWrapper {}
unsafe impl Sync for FuncWrapper {}

impl FuncWrapper {
    const fn new() -> Self {
        Self(Cell::new(None))
    }

    fn set(&self, e: Entry) {
        self.0.set(Some(e));
    }

    fn get(&self) -> Option<Entry> {
        self.0.take()
    }
}
