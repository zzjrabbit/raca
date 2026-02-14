use alloc::{
    collections::vec_deque::VecDeque,
    sync::{Arc, Weak},
};
use spin::Mutex;

use crate::task::HwThread;

pub static SCHEDULER: Mutex<Scheduler> = Mutex::new(Scheduler::new());

pub struct Scheduler {
    threads: VecDeque<Arc<HwThread>>,
    current: Option<Weak<HwThread>>,
}

impl Scheduler {
    const fn new() -> Self {
        Self {
            threads: VecDeque::new(),
            current: None,
        }
    }
}

impl Scheduler {
    pub fn get_next(&mut self) -> Arc<HwThread> {
        let next = self.threads.pop_front().unwrap();
        self.current = Some(Arc::downgrade(&next));
        next
    }

    pub fn current(&self) -> Option<Arc<HwThread>> {
        self.current.clone().and_then(|t| t.upgrade())
    }

    pub fn remove(&mut self, thread: &Arc<HwThread>) {
        self.threads.retain(|t| !Arc::ptr_eq(thread, t));
    }

    pub fn add(&mut self, thread: &Arc<HwThread>) {
        self.threads.push_back(thread.clone());
    }
}
