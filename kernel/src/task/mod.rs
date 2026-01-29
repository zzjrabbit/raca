use core::{
    cell::{Cell, SyncUnsafeCell},
    sync::atomic::{AtomicBool, Ordering},
};

use alloc::{boxed::Box, sync::Arc};
use spin::Mutex;

pub use crate::arch::context::UserContext;
use crate::arch::context::{
    TaskContext as TaskContextImpl, context_switch, first_context_switch, kernel_task_entry_wrapper,
};

pub trait TaskContextApi {
    unsafe fn set_program_counter(&mut self, pc: usize);
    unsafe fn set_stack_pointer(&mut self, sp: usize);
}

static CURRENT_TASK_CONTEXT: Mutex<Option<Arc<TaskContext>>> = Mutex::new(None);

pub struct TaskContext {
    inner: SyncUnsafeCell<TaskContextImpl>,
    func: Cell<Option<Box<dyn FnOnce() + Send>>>,
    already_running: AtomicBool,
}

unsafe impl Send for TaskContext {}
unsafe impl Sync for TaskContext {}

impl TaskContext {
    pub fn current() -> Option<Arc<TaskContext>> {
        CURRENT_TASK_CONTEXT.lock().clone()
    }
}

fn alloc_stack() -> &'static mut [u8] {
    Box::leak(Box::new([0u8; 8 * 1024]))
}

pub(crate) extern "C" fn kernel_task_entry() -> ! {
    let current = TaskContext::current().unwrap();
    (current.func.take().unwrap())();
    loop {}
}

impl TaskContext {
    pub fn new<F>(func: F) -> Arc<Self>
    where
        F: FnOnce() + Send + 'static,
    {
        let mut context = TaskContextImpl::new();

        unsafe {
            context.set_program_counter(kernel_task_entry_wrapper as *const () as usize);
            context.set_stack_pointer(alloc_stack().as_ptr() as usize);
        }

        let context = SyncUnsafeCell::new(context);
        let context = TaskContext {
            inner: context,
            func: Cell::new(Some(Box::new(func))),
            already_running: AtomicBool::new(false),
        };
        Arc::new(context)
    }
}

impl TaskContextApi for TaskContext {
    unsafe fn set_program_counter(&mut self, pc: usize) {
        unsafe {
            self.inner.get_mut().set_program_counter(pc);
        }
    }

    unsafe fn set_stack_pointer(&mut self, sp: usize) {
        unsafe {
            self.inner.get_mut().set_stack_pointer(sp);
        }
    }
}

impl TaskContext {
    fn prepare_to_run(&self) {
        if self.already_running.load(Ordering::SeqCst) {
            log::warn!("Switching to task already running in foreground!");
            while self.already_running.load(Ordering::SeqCst) {
                core::hint::spin_loop();
            }
        }
    }

    fn running(&self) -> bool {
        self.already_running.load(Ordering::SeqCst)
    }

    fn toggle_running(&self) {
        self.already_running.fetch_not(Ordering::SeqCst);
    }
}

impl TaskContext {
    pub fn save_and_load(&self, other: &Arc<TaskContext>) {
        if !self.running() {
            panic!("Saving context to a non-running task!");
        }

        other.prepare_to_run();
        self.toggle_running();
        other.toggle_running();

        CURRENT_TASK_CONTEXT.lock().replace(other.clone());

        unsafe {
            context_switch(other.inner.get() as *const _, self.inner.get());
        }
    }

    pub fn load_without_save(self: &Arc<TaskContext>) {
        self.prepare_to_run();
        self.toggle_running();

        CURRENT_TASK_CONTEXT.lock().replace(self.clone());

        unsafe {
            first_context_switch(self.inner.get() as *const _);
        }
    }
}
