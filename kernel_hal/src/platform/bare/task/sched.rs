use alloc::{
    collections::vec_deque::VecDeque,
    sync::{Arc, Weak},
};
use spin::Mutex;

use crate::task::HwThread;

pub static SCHEDULER: Mutex<Scheduler> = Mutex::new(Scheduler::new());

#[derive(Debug)]
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
    pub fn no_next(&self) -> bool {
        self.threads.is_empty()
    }

    pub fn get_next(&mut self) -> Option<Arc<HwThread>> {
        let next = self.threads.pop_front()?;
        self.current = Some(Arc::downgrade(&next));
        Some(next)
    }

    pub fn current(&self) -> Option<Arc<HwThread>> {
        self.current.clone().and_then(|t| t.upgrade())
    }

    pub fn take_current(&mut self) -> Option<Arc<HwThread>> {
        self.current.take().and_then(|t| t.upgrade())
    }

    pub fn remove(&mut self, thread: &Arc<HwThread>) {
        self.threads.retain(|t| !Arc::ptr_eq(thread, t));
    }

    pub fn add(&mut self, thread: &Arc<HwThread>) {
        self.threads.push_back(thread.clone());
    }
}
