use alloc::vec::Vec;
use errors::Result;
use protocol::{PROC_HANDLE_IDX, PROC_START_HANDLE_CNT, ProcessStartInfo, VMAR_HANDLE_IDX};

use crate::{
    ipc::{Channel, MessagePacket},
    os::raca::{BorrowedHandle, OwnedHandle},
    syscall::{sys_exit, sys_kill_process, sys_new_process, sys_new_thread, sys_start_process},
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

    pub(crate) fn handle(&self) -> BorrowedHandle {
        self.handle.borrow()
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
    pub fn start(&self, thread: &Thread, _binary: &[u8]) -> Result<()> {
        let (channel0, channel1) = Channel::new()?;

        let mut handles = alloc::vec![unsafe {OwnedHandle::from_raw(0)}; PROC_START_HANDLE_CNT];
        handles[PROC_HANDLE_IDX] = self.handle().duplicate();
        handles[VMAR_HANDLE_IDX] = self.vmar().handle().duplicate();
        channel1.write(MessagePacket {
            data: Vec::new(),
            handles,
        })?;

        unsafe {
            sys_start_process(
                self.handle.as_raw(),
                thread.handle().as_raw(),
                channel0.0.as_raw(),
                0,
                0,
                &ProcessStartInfo {
                    vmar_base: self.vmar().base(),
                    vmar_size: self.vmar().size(),
                },
            )?;
        }

        Ok(())
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
