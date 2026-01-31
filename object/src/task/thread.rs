use core::sync::atomic::{AtomicU64, Ordering};

use alloc::sync::Arc;
use spin::Mutex;

use crate::{impl_kobj, new_kobj, object::KObjectBase};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ThreadId(u64);

impl ThreadId {
    pub fn new() -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);
        Self(NEXT_ID.fetch_add(1, Ordering::SeqCst))
    }
}

pub struct Thread {
    inner: Mutex<ThreadInner>,
    tid: ThreadId,
    base: KObjectBase,
}

struct ThreadInner {}

impl_kobj!(Thread);

impl Thread {
    pub fn new() -> Arc<Self> {
        new_kobj!({
            inner: Mutex::new(ThreadInner {  }),
            tid: ThreadId::new(),
        })
    }
}

impl Thread {
    pub fn id(&self) -> ThreadId {
        self.tid
    }
}
