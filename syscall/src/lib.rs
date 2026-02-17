#![no_std]

extern crate alloc;

use alloc::{string::String, sync::Arc};
use errors::{Errno, Result};
use kernel_hal::task::UserContext;
use object::task::Process;

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
        0 => {
            let mut buf = alloc::vec![0u8; arg2];
            process.root_vmar().read(arg1, &mut buf)?;
            let Ok(msg) = String::from_utf8(buf) else {
                return Err(Errno::InvArg.no_message());
            };
            log::info!("USER DEBUG: {}", msg);
        }
        _ => {}
    }
    Ok(0)
}
