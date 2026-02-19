use errors::Result;

use crate::{
    os::raca::OwnedHandle,
    syscall::{sys_exit, sys_kill_process, sys_new_process, sys_new_thread},
    thread::Thread,
    vm::Vmar,
};

pub struct Process {
    handle: OwnedHandle,
    root_vmar: Vmar,
}

impl Process {
    pub unsafe fn from_handle_vmar(handle: OwnedHandle, root_vmar: Vmar) -> Self {
        Self { handle, root_vmar }
    }

    pub fn new() -> Result<Self> {
        let mut raw_handle = 0;
        let mut raw_vmar_handle = 0;
        let mut base = 0;
        let mut size = 0;
        unsafe {
            sys_new_process(&mut raw_handle, &mut raw_vmar_handle, &mut base, &mut size).unwrap();
        }
        let handle = unsafe { OwnedHandle::from_raw(raw_handle) };
        let root_vmar = unsafe {
            Vmar::from_handle_base_size(OwnedHandle::from_raw(raw_vmar_handle), base, size)
        };
        Ok(Self { handle, root_vmar })
    }
}

impl Process {
    pub fn vmar(&self) -> &Vmar {
        &self.root_vmar
    }
}

impl Process {
    pub fn new_thread(&self) -> Result<Thread> {
        let mut raw_handle = 0;
        unsafe {
            sys_new_thread(self.handle.as_raw(), &mut raw_handle).unwrap();
        }
        let handle = unsafe { OwnedHandle::from_raw(raw_handle) };
        Ok(unsafe { Thread::from_handle(handle) })
    }
}

impl Process {
    pub fn kill(&self) -> Result<()> {
        unsafe {
            sys_kill_process(self.handle.as_raw()).unwrap();
        }
        Ok(())
    }
}

pub fn exit(exit_code: i32) -> ! {
    unsafe {
        sys_exit(exit_code).unwrap();
    }
    #[allow(clippy::empty_loop)]
    loop {}
}
