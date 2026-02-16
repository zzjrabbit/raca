use alloc::{collections::btree_map::BTreeMap, sync::Arc, vec::Vec};
use kernel_hal::{mem::VirtAddr, task::UserContext};
use spin::lock_api::Mutex;

use crate::{
    Errno, Result, impl_kobj,
    mem::Vmar,
    new_kobj,
    object::{Handle, KObjectBase, KernelObject, Rights},
    task::{Thread, exception::page_fault_handler},
};

mod vdso;

pub struct Process {
    inner: Mutex<ProcessInner>,
    vmar: Arc<Vmar>,
    vdso: Arc<Vmar>,
    vdso_load_base: VirtAddr,
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
        let vmar = Vmar::new_root();
        let (vdso_load_base, vdso) = Self::map_vdso(vmar.clone());
        new_kobj!({
            inner: Mutex::new(ProcessInner {
                threads: Vec::new(),
                handles: BTreeMap::new(),
            }),
            vmar,
            vdso,
            vdso_load_base,
        })
    }
}

impl Process {
    pub fn root_vmar(&self) -> &Arc<Vmar> {
        &self.vmar
    }

    pub fn vdso(&self) -> &Arc<Vmar> {
        &self.vdso
    }

    pub fn vdso_load_base(&self) -> VirtAddr {
        self.vdso_load_base
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
        let mut user_ctx = UserContext::default();
        user_ctx.set_ip(entry);
        user_ctx.set_sp(stack);

        initializer(&mut user_ctx);

        let process = self.clone();
        thread.start(move || {
            process.root_vmar().activate();
            let info = user_ctx.enter_user_space();
            #[cfg(not(feature = "libos"))]
            if matches!(info.code, 1..8) {
                if page_fault_handler(&info).is_err() {
                    log::error!("page fault handler failed");
                    kernel_hal::platform::idle_loop();
                }
            } else if info.code == 0xb {
                syscall_handler(&process, &mut user_ctx);
            } else {
                log::error!("unknown exception code: {}", info.code);
                kernel_hal::platform::idle_loop();
            }
        });
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
    use kernel_hal::{mem::PageProperty, task::ThreadState};

    use crate::{mem::Vmo, object::Upcast};

    use super::*;

    extern crate std;

    #[test]
    fn proc_handle() {
        let proc = Process::new();
        let handle = proc.add_handle(Handle::new(proc.clone().upcast(), Rights::READ));
        assert_eq!(handle, HandleId(0));
        proc.remove_handle(handle);
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
