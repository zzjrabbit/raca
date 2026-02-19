use alloc::{collections::btree_map::BTreeMap, sync::Arc, vec::Vec};
use kernel_hal::{mem::VirtAddr, task::UserContext};
use pod::derive;
use spin::lock_api::Mutex;

use crate::{
    Errno, Result, impl_kobj,
    mem::Vmar,
    new_kobj,
    object::{Handle, KObjectBase, KernelObject, Rights},
    task::Thread,
};

pub struct Process {
    inner: Mutex<ProcessInner>,
    vmar: Arc<Vmar>,
    base: KObjectBase,
}

struct ProcessInner {
    threads: Vec<Arc<Thread>>,
    handles: BTreeMap<HandleId, Handle>,
    exit_status: Option<i32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Pod)]
#[repr(transparent)]
pub struct HandleId(u32);

impl HandleId {
    pub fn from_raw(raw: u32) -> Self {
        Self(raw)
    }

    pub fn as_raw(&self) -> u32 {
        self.0
    }
}

impl_kobj!(Process);

impl Process {
    pub fn new() -> Arc<Self> {
        let vmar = Vmar::new_root();
        new_kobj!({
            inner: Mutex::new(ProcessInner {
                threads: Vec::new(),
                handles: BTreeMap::new(),
                exit_status: None,
            }),
            vmar,
        })
    }
}

impl Process {
    pub fn root_vmar(&self) -> &Arc<Vmar> {
        &self.vmar
    }

    pub fn exit_status(&self) -> Option<i32> {
        let inner = self.inner.lock();
        inner.exit_status
    }
}

impl Process {
    pub fn current() -> Option<Arc<Self>> {
        Thread::current().and_then(|thread| thread.process())
    }
}

impl Process {
    pub fn new_thread(self: &Arc<Self>) -> Arc<Thread> {
        let thread = Thread::new(Arc::downgrade(self));
        self.add_thread(thread.clone());
        thread
    }
}

impl Process {
    pub fn start(
        self: &Arc<Self>,
        thread: Arc<Thread>,
        entry: VirtAddr,
        stack: usize,
        initializer: impl FnOnce(&mut UserContext),
        syscall_handler: impl Fn(&Arc<Self>, &mut UserContext) + Send + 'static,
    ) {
        thread.start_user(self.clone(), entry, stack, initializer, syscall_handler);
    }

    pub fn exit(&self, status: i32) {
        let current_thread = Thread::current().unwrap();
        {
            for thread in self.inner.lock().threads.clone() {
                if thread.id() != current_thread.id() {
                    thread.kill();
                }
            }
            let mut inner = self.inner.lock();
            inner.exit_status = Some(status);
        }
        current_thread.exit();
    }

    pub fn kill(&self) {
        {
            let mut inner = self.inner.lock();
            inner.exit_status = Some(-1);
        }
        for thread in self.inner.lock().threads.clone() {
            thread.kill();
        }
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

    pub fn remove_handle(&self, id: HandleId) -> Result<Handle> {
        let mut inner = self.inner.lock();
        inner
            .handles
            .remove(&id)
            .ok_or(Errno::BadHandle.with_message("Handle not found!"))
    }

    pub fn remove_handle_with_rights(
        &self,
        id: HandleId,
        desired_rights: Rights,
    ) -> Result<Handle> {
        let mut inner = self.inner.lock();
        let handle = inner
            .handles
            .remove(&id)
            .ok_or(Errno::BadHandle.with_message("Handle not found!"))?;
        if handle.rights.contains(desired_rights) {
            Ok(handle)
        } else {
            Err(Errno::AccessDenied.with_message("Handle does not have the desired rights!"))
        }
    }

    pub fn add_thread(&self, thread: Arc<Thread>) {
        let mut inner = self.inner.lock();
        inner.threads.push(thread);
    }

    pub fn remove_thread(&self, thread: &Arc<Thread>) {
        let mut inner = self.inner.lock();
        inner.threads.retain(|t| t.id() != thread.id());
        if inner.threads.is_empty() {
            inner.exit_status = Some(0);
        }
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
    use kernel_hal::{mem::PageProperty, task::ThreadState};

    use crate::{mem::Vmo, object::Upcast};

    use super::*;

    extern crate std;

    #[test]
    fn proc_handle() {
        let proc = Process::new();
        let handle = proc.add_handle(Handle::new(proc.clone().upcast(), Rights::READ));
        assert_eq!(handle, HandleId(0));
        proc.remove_handle(handle).unwrap();
    }

    #[test]
    fn proc_start() {
        const STACK_SIZE: usize = 8 * 1024 * 1024;

        extern "C" fn user_entry(_vmo_start: VirtAddr) {
            loop {}
        }

        let process = Process::new();
        let thread = process.new_thread();

        let stack = process.root_vmar().allocate_child(STACK_SIZE).unwrap();
        stack
            .map(
                0,
                &Vmo::allocate_ram(stack.page_count()).unwrap(),
                PageProperty::user_data(),
                false,
            )
            .unwrap();

        process.start(
            thread.clone(),
            user_entry as *const () as VirtAddr,
            stack.end(),
            |_| {},
            |_, _| {},
        );
        std::thread::sleep(std::time::Duration::from_millis(100));
        thread.set_state(ThreadState::Blocked);
    }
}
