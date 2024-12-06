use core::arch::naked_asm;

use x86_64::{
    VirtAddr,
    registers::{
        control::{Efer, EferFlags},
        model_specific::{LStar, SFMask, Star},
        rflags::RFlags,
    },
};

use crate::{arch::gdt::Selectors, error::Error};

mod consts;
mod debug;
mod fs;
mod kernel;
mod memory;
mod task;

use consts::SyscallIndex;
use debug::*;
use fs::*;
use kernel::*;
use memory::*;
use task::*;

#[naked]
extern "C" fn asm_syscall_handler() {
    unsafe {
        naked_asm!(
            "push rcx",
            "push r11",
            "push rbp",
            "push rbx",
            "push r12",
            "push r13",
            "push r14",
            "push r15",

            "mov rcx, r10",

            "call {syscall_matcher}",

            "pop r15",
            "pop r14",
            "pop r13",
            "pop r12",
            "pop rbx",
            "pop rbp",
            "pop r11",
            "pop rcx",
            "sysretq",
            syscall_matcher = sym syscall_handler,
        );
    }
}

pub fn init() {
    let handler_addr = asm_syscall_handler as *const () as u64;

    SFMask::write(RFlags::INTERRUPT_FLAG);
    LStar::write(VirtAddr::new(handler_addr as u64));

    let (code_selector, data_selector) = Selectors::get_kernel_segments();
    let (user_code_selector, user_data_selector) = Selectors::get_user_segments();

    Star::write(
        user_code_selector,
        user_data_selector,
        code_selector,
        data_selector,
    )
    .unwrap();

    unsafe {
        Efer::write(Efer::read() | EferFlags::SYSTEM_CALL_EXTENSIONS);
    }
}

#[allow(unused_variables)]
pub extern "C" fn syscall_handler(
    syscall_number_raw: usize,
    arg1: usize,
    arg2: usize,
    arg3: usize,
    arg4: usize,
    arg5: usize,
) -> isize {
    //log::info!("number : {}", syscall_number_raw);

    let sys_type = match SyscallIndex::try_from(syscall_number_raw) {
        Ok(index) => index,
        Err(_) => return Error::InvalidSyscall as isize,
    };

    let ret = match sys_type {
        SyscallIndex::Debug => debug(arg1, arg2),
        SyscallIndex::Open => open(arg1, arg2, arg3),
        SyscallIndex::Malloc => malloc(arg1, arg2),
        SyscallIndex::Read => read(arg1, arg2, arg3),
        SyscallIndex::Write => write(arg1, arg2, arg3),
        SyscallIndex::Lseek => lseek(arg1, arg2),
        SyscallIndex::Close => close(arg1),
        SyscallIndex::Fsize => fsize(arg1),
        SyscallIndex::CreateProcess => create_process(arg1),
        SyscallIndex::InsertModule => insert_module(arg1, arg2),
        SyscallIndex::HasSignal => has_signal(arg1),
        SyscallIndex::GetSignal => get_signal(arg1, arg2),
        SyscallIndex::DoneSignal => done_signal(arg1),
        SyscallIndex::StartWaitForSignal => start_wait_for_signal(arg1),
        SyscallIndex::Exit => exit(arg1),
        SyscallIndex::PowerOff => poweroff(),
        SyscallIndex::Reboot => reboot(),
        SyscallIndex::CreateThread => create_thread(arg1, arg2 as u64),
        SyscallIndex::YieldProcess => yield_thread(),
        SyscallIndex::Sleep => sleep(arg1),
    };

    match ret {
        Ok(num) => num as isize,
        Err(err) => err as isize,
    }
}
