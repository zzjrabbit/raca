use alloc::collections::btree_map::BTreeMap;
use spin::lock_api::Mutex;

use crate::{
    Errno, Result,
    object::{Handle, KernelObject, Rights, TypedKObject},
};

pub struct Process {
    inner: Mutex<ProcessInner>,
}

struct ProcessInner {
    handles: BTreeMap<HandleId, Handle>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct HandleId(u32);

impl KernelObject for Process {}

impl Process {
    pub fn new() -> TypedKObject<Self> {
        TypedKObject::new(Self {
            inner: Mutex::new(ProcessInner {
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
}

impl Process {
    pub fn find_object_with_rights<T: KernelObject>(
        &self,
        handle_id: HandleId,
        desired_rights: Rights,
    ) -> Result<TypedKObject<T>> {
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
                .downcast::<T>()
                .ok_or(Errno::WrongType.no_message())
        } else {
            Err(Errno::AccessDenied.no_message())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn proc_handle() {
        let proc = Process::new();
        let handle = proc.add_handle(Handle::new(proc.clone().into(), Rights::READ));
        assert_eq!(handle, HandleId(0));
        proc.remove_handle(handle);
    }
}
