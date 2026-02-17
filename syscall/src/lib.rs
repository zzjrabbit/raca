#![no_std]

extern crate alloc;

use alloc::sync::Arc;
use errors::{Errno, Result};
use kernel_hal::task::UserContext;
use object::task::Process;

use crate::{debug::debug, ipc::new_channel};

mod debug;
mod ipc;

type SyscallResult = Result<usize>;

pub fn syscall_handler(process: &Arc<Process>, user_ctx: &mut UserContext) {
    let [arg1, arg2, arg3, arg4, arg5, arg6] = user_ctx.get_syscall_args();
    let id = user_ctx.get_syscall_num();

    let result = match syscall_impl(process, user_ctx) {
        Ok(ret) => ret as isize,
        Err(err) => {
            log::warn!("syscall error: {}", err);
            i32::from(err) as isize
        }
    };

    log::debug!(
        "syscall{}({:#x}, {:#x}, {:#x}, {:#x}, {:#x}, {:#x}) -> {:#x}",
        id,
        arg1,
        arg2,
        arg3,
        arg4,
        arg5,
        arg6,
        result
    );

    user_ctx.set_syscall_ret(result as usize);
}

fn syscall_impl(process: &Arc<Process>, user_ctx: &mut UserContext) -> Result<usize> {
    #[allow(unused_variables)]
    let [arg1, arg2, arg3, arg4, arg5, arg6] = user_ctx.get_syscall_args();
    let id = user_ctx.get_syscall_num();

    match id {
        0 => debug(process, arg1, arg2),
        1 => new_channel(process, arg1, arg2),
        _ => Err(Errno::InvSyscall.no_message()),
    }
}
