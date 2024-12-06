use core::sync::atomic::{AtomicU64, Ordering};

use crate::{
    arch::gdt::Selectors,
    memory::{ExtendedPageTable, KERNEL_PAGE_TABLE},
};

use super::{SCHEDULER, context::Context, process::KERNEL_PROCESS, stack::UserStack};
use super::{process::WeakSharedProcess, stack::KernelStack};
use alloc::{
    boxed::Box,
    sync::{Arc, Weak},
};
use spin::RwLock;
use x86_64::instructions::interrupts;

pub(super) type SharedThread = Arc<RwLock<Box<Thread>>>;
pub(super) type WeakSharedThread = Weak<RwLock<Box<Thread>>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ThreadId(pub u64);

impl ThreadId {
    fn new() -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);
        ThreadId(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }
}

pub struct Thread {
    pub id: ThreadId,
    pub kernel_stack: KernelStack,
    pub context: Context,
    pub process: WeakSharedProcess,
    pub waiter: Option<WeakSharedThread>,
    pub remove_after_schedule: bool,
}

impl Thread {
    pub(self) fn new(process: WeakSharedProcess) -> Self {
        Thread {
            id: ThreadId::new(),
            kernel_stack: KernelStack::new(),
            context: Context::default(),
            process,
            waiter: None,
            remove_after_schedule: false,
        }
    }

    pub fn get_init_thread() -> WeakSharedThread {
        let thread = Self::new(Arc::downgrade(&KERNEL_PROCESS));
        let thread = Arc::new(RwLock::new(Box::new(thread)));
        KERNEL_PROCESS.write().threads.push(thread.clone());
        Arc::downgrade(&thread)
    }

    pub fn new_kernel_thread(function: fn()) {
        let mut thread = Self::new(Arc::downgrade(&KERNEL_PROCESS));

        thread.context.init(
            function as usize,
            thread.kernel_stack.end_address(),
            KERNEL_PAGE_TABLE.lock().physical_address(),
            Selectors::get_kernel_segments(),
        );

        let thread = Arc::new(RwLock::new(Box::new(thread)));
        KERNEL_PROCESS.write().threads.push(thread.clone());

        interrupts::without_interrupts(|| {
            SCHEDULER.lock().add(Arc::downgrade(&thread));
        });
    }

    pub fn new_user_thread(process: WeakSharedProcess, entry_point: usize) -> SharedThread {
        let mut thread = Self::new(process.clone());
        let process = process.upgrade().unwrap();
        let mut process = process.write();
        let user_stack = UserStack::new(&mut process.page_table);

        thread.context.init(
            entry_point,
            user_stack.end_address,
            process.page_table.physical_address(),
            Selectors::get_user_segments(),
        );

        let thread = Arc::new(RwLock::new(Box::new(thread)));
        process.threads.push(thread.clone());

        SCHEDULER.lock().add(Arc::downgrade(&thread.clone()));
        thread
    }

    pub fn new_user_thread_with_stack(
        process: WeakSharedProcess,
        entry_point: usize,
        stack_end: x86_64::VirtAddr,
    ) -> SharedThread {
        let mut thread = Self::new(process.clone());
        let process = process.upgrade().unwrap();
        let mut process = process.write();

        thread.context.init(
            entry_point,
            stack_end,
            process.page_table.physical_address(),
            Selectors::get_user_segments(),
        );

        let thread = Arc::new(RwLock::new(Box::new(thread)));
        process.threads.push(thread.clone());

        SCHEDULER.lock().add(Arc::downgrade(&thread.clone()));
        thread
    }
}
