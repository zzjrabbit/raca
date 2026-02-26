#![no_std]

extern crate alloc;

use core::fmt::Debug;

use alloc::sync::Arc;
use errors::{Errno, Result};
use kernel_hal::task::UserContext;
use object::task::Process;

use crate::{
    debug::debug,
    handle::{duplicate_handle, remove_handle},
    ipc::{new_channel, read_channel, write_channel},
    task::{
        exit, exit_thread, kill_process, kill_thread, new_process, new_thread, start_process,
        start_thread,
    },
    vm::{
        acquire_vmo, allocate_vmar, allocate_vmar_at, allocate_vmo, get_vmar_base, get_vmar_size,
        map_vmar, protect_vmar, read_vmo, unmap_vmar, write_vmo,
    },
};

mod debug;
mod handle;
mod ipc;
mod task;
mod vm;

type SyscallResult = Result<usize>;

pub fn syscall_handler(process: &Arc<Process>, user_ctx: &mut UserContext) {
    let [arg1, arg2, arg3, arg4, arg5, arg6] = user_ctx.get_syscall_args();
    let id = user_ctx.get_syscall_num();

    let result = syscall_impl(process, user_ctx);

    if id != 0 {
        log::debug!(
            "process{}({:#x}) syscall{}({:#x}, {:#x}, {:#x}, {:#x}, {:#x}, {:#x}) -> {:?}",
            process.id(),
            user_ctx.get_ip(),
            id,
            arg1,
            arg2,
            arg3,
            arg4,
            arg5,
            arg6,
            WrappedResult(&result),
        );
    }

    let raw_result = match result {
        Ok(ret) => ret,
        Err(err) => {
            log::warn!("syscall error: {}", err);
            i32::from(err) as usize
        }
    };

    user_ctx.set_syscall_ret(raw_result);
}

fn syscall_impl(process: &Arc<Process>, user_ctx: &mut UserContext) -> Result<usize> {
    #[allow(unused_variables)]
    let [arg1, arg2, arg3, arg4, arg5, arg6] = user_ctx.get_syscall_args();
    let id = user_ctx.get_syscall_num();

    match id {
        0 => debug(process, arg1, arg2),
        1 => remove_handle(process, arg1 as u32),
        2 => new_channel(process, arg1, arg2),
        3 => read_channel(process, arg1 as u32, arg2, arg3),
        4 => write_channel(process, arg1 as u32, arg2, arg3),
        5 => allocate_vmar(process, arg1 as u32, arg2, arg3),
        6 => allocate_vmar_at(process, arg1 as u32, arg2, arg3, arg4),
        7 => map_vmar(process, arg1 as u32, arg2, arg3 as u32, arg4 as u32),
        8 => unmap_vmar(process, arg1 as u32, arg2, arg3),
        9 => protect_vmar(process, arg1 as u32, arg2, arg3, arg4 as u32),
        10 => allocate_vmo(process, arg1, arg2),
        11 => exit(process, arg1 as i32),
        12 => new_process(process, arg1, arg2, arg3, arg4),
        13 => start_process(
            process,
            arg1 as u32,
            arg2 as u32,
            arg3 as u32,
            arg4,
            arg5,
            arg6,
        ),
        14 => new_thread(process, arg1 as u32, arg2),
        15 => start_thread(process, arg1 as u32, arg2, arg3, arg4),
        16 => exit_thread(),
        17 => kill_process(process, arg1 as u32),
        18 => kill_thread(process, arg1 as u32),
        19 => duplicate_handle(process, arg1 as u32, arg2),
        20 => read_vmo(process, arg1 as u32, arg2, arg3, arg4),
        21 => write_vmo(process, arg1 as u32, arg2, arg3, arg4),
        22 => get_vmar_base(process, arg1 as u32),
        23 => get_vmar_size(process, arg1 as u32),
        24 => acquire_vmo(process, arg1, arg2, arg3),
        _ => Err(Errno::InvSyscall.no_message()),
    }
}

struct WrappedResult<'a>(&'a Result<usize>);

impl Debug for WrappedResult<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match &self.0 {
            Ok(ret) => write!(f, "{:#x}", ret),
            Err(err) => write!(f, "Err({})", err),
        }
    }
}
