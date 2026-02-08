use alloc::{collections::btree_map::BTreeMap, sync::Arc, vec::Vec};
use spin::lock_api::Mutex;

use crate::{
    Errno, Result, impl_kobj, new_kobj,
    object::{Handle, KObjectBase, KernelObject, Rights},
    task::Thread,
};

#[allow(unused)]
static VDSO: &'static [u8] = include_bytes!(concat!("../../../", env!("VDSO_PATH")));

pub struct Process {
    inner: Mutex<ProcessInner>,
    base: KObjectBase,
}

struct ProcessInner {
    threads: Vec<Arc<Thread>>,
    handles: BTreeMap<HandleId, Handle>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct HandleId(u32);

impl_kobj!(Process);

impl Process {
    pub fn new() -> Arc<Self> {
        new_kobj!({
            inner: Mutex::new(ProcessInner {
                threads: Vec::new(),
                handles: BTreeMap::new(),
            }),
        })
    }
}

impl Process {
    pub fn add_handle(&self, handle: Handle) -> HandleId {
        let mut inner = self.inner.lock();
        let id = HandleId(
            (0u32..)
                .find(|idx| !inner.handles.contains_key(&HandleId(*idx)))
                .unwrap(),
        );
        inner.handles.insert(id, handle);
        id
    }

    pub fn remove_handle(&self, id: HandleId) {
        let mut inner = self.inner.lock();
        inner.handles.remove(&id);
    }

    pub fn add_thread(&self, thread: Arc<Thread>) {
        let mut inner = self.inner.lock();
        inner.threads.push(thread);
    }

    pub fn remove_thread(&self, thread: Arc<Thread>) {
        let mut inner = self.inner.lock();
        inner.threads.retain(|t| t.id() != thread.id());
    }
}

impl Process {
    pub fn find_object_with_rights<T: KernelObject>(
        &self,
        handle_id: HandleId,
        desired_rights: Rights,
    ) -> Result<Arc<T>> {
        let handle = self
            .inner
            .lock()
            .handles
            .get(&handle_id)
            .ok_or(Errno::BadHandle.with_message("Handle not found!"))?
            .clone();
        if handle.rights.contains(desired_rights) {
            handle
                .object
                .downcast_arc::<T>()
                .map_err(|_| Errno::WrongType.no_message())
        } else {
            Err(Errno::AccessDenied.no_message())
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::object::Upcast;

    use super::*;

    #[test]
    fn proc_handle() {
        let proc = Process::new();
        let handle = proc.add_handle(Handle::new(proc.clone().upcast(), Rights::READ));
        assert_eq!(handle, HandleId(0));
        proc.remove_handle(handle);
    }
}
