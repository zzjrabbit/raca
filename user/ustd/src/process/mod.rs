use alloc::vec::Vec;
use errors::Result;
use protocol::{PROC_HANDLE_IDX, PROC_START_HANDLE_CNT, ProcessStartInfo, VMAR_HANDLE_IDX};
use spin::Once;

use crate::{
    ipc::{Channel, MessagePacket},
    os::raca::{BorrowedHandle, OwnedHandle},
    process::{
        loader::load_elf,
        stack::{new_user_stack, push_stack},
    },
    syscall::{sys_exit, sys_kill_process, sys_new_process, sys_new_thread, sys_start_process},
    thread::Thread,
    vm::Vmar,
};

mod loader;
mod stack;

static CURRENT_PROCESS: Once<Process> = Once::new();

pub(crate) fn init(process: Process) {
    CURRENT_PROCESS.call_once(|| process);
}

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

    pub fn current() -> &'static Self {
        CURRENT_PROCESS.get().unwrap()
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
    pub fn start(&self, thread: &Thread, binary: &[u8]) -> Result<()> {
        let (channel0, channel1) = Channel::new()?;

        let entry = load_elf(self.vmar(), binary)?;
        crate::println!("entry {:#x}", entry);

        let (stack_vmo, stack) = new_user_stack(self.vmar())?;
        let mut stack_ptr = stack.end();

        let proc_info = ProcessStartInfo {
            vmar_base: self.vmar().base(),
            vmar_size: self.vmar().size(),
        };
        let proc_info_addr = push_stack(&stack, &stack_vmo, &mut stack_ptr, &proc_info)?;
        crate::println!("Proc Info: {:#x?} at {:#x}", proc_info, proc_info_addr);

        let mut handles = alloc::vec![unsafe {OwnedHandle::from_raw(0)}; PROC_START_HANDLE_CNT];
        handles[PROC_HANDLE_IDX] = self.handle().duplicate();
        handles[VMAR_HANDLE_IDX] = self.vmar().handle().duplicate();
        channel1.write(MessagePacket {
            data: Vec::new(),
            handles,
        })?;

        crate::println!(
            "stack start at {:#x} end at {:#x}",
            stack.base(),
            stack.end()
        );
        unsafe {
            sys_start_process(
                self.handle.as_raw(),
                thread.handle().as_raw(),
                channel0.0.as_raw(),
                entry,
                stack_ptr,
                proc_info_addr,
            )?;
        }

        core::mem::forget(channel0);

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
