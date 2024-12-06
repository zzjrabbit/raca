use core::sync::atomic::{AtomicBool, Ordering};

use alloc::{
    collections::{btree_map::BTreeMap, vec_deque::VecDeque},
    sync::Arc,
};
use context::Context;
use spin::{Lazy, Mutex};
use x86_64::VirtAddr;

use crate::arch::{apic::LAPIC, smp::CPUS};

use super::*;

pub static SCHEDULER_INIT: AtomicBool = AtomicBool::new(false);
pub static SCHEDULER: Lazy<Mutex<Scheduler>> = Lazy::new(|| Mutex::new(Scheduler::new()));

pub fn init() {
    x86_64::instructions::interrupts::enable();
    SCHEDULER_INIT.store(true, Ordering::SeqCst);
    log::info!("Scheduler initialized, interrupts enabled!");
}

pub struct Scheduler {
    current_threads: BTreeMap<u32, WeakSharedThread>,
    ready_threads: VecDeque<WeakSharedThread>,
}

impl Scheduler {
    pub fn new() -> Self {
        let current_threads = CPUS
            .read()
            .iter_id()
            .map(|lapic_id| (*lapic_id, Thread::get_init_thread()))
            .collect();

        Self {
            current_threads,
            ready_threads: VecDeque::new(),
        }
    }

    #[inline]
    pub fn add(&mut self, thread: WeakSharedThread) {
        self.ready_threads.push_back(thread);
    }

    #[inline]
    pub fn remove(&mut self, thread: WeakSharedThread) {
        self.ready_threads.retain(|other| {
            let other = other.upgrade().unwrap();
            !Arc::ptr_eq(&other, &thread.upgrade().unwrap())
        });
    }

    #[inline]
    pub fn current_thread(&self) -> WeakSharedThread {
        let lapic_id = unsafe { LAPIC.lock().id() };
        self.current_threads[&lapic_id].clone()
    }

    pub fn schedule(&mut self, context: VirtAddr) -> VirtAddr {
        let lapic_id = unsafe { LAPIC.lock().id() };

        let last_thread = self.current_threads.get(&lapic_id).and_then(|thread| {
            thread.upgrade().and_then(|thread| {
                thread.write().context = Context::from_address(context);
                Some(self.current_threads[&lapic_id].clone())
            })
        });

        if let Some(next_thread) = self.ready_threads.pop_front() {
            self.current_threads.insert(lapic_id, next_thread);
            if let Some(last_thread) = last_thread {
                let is_going_to_remove = last_thread.upgrade().unwrap().read().remove_after_schedule;
                last_thread.upgrade().unwrap().write().remove_after_schedule = false;
                if !is_going_to_remove {
                    self.ready_threads.push_back(last_thread);
                }
            }
        }

        let next_thread = self.current_threads[&lapic_id].upgrade().unwrap();
        let next_thread = next_thread.read();

        let kernel_address = next_thread.kernel_stack.end_address();
        CPUS.write().get_mut(lapic_id).set_ring0_rsp(kernel_address);

        next_thread.context.address()
    }
}
