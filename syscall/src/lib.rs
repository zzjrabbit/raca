#![no_std]

extern crate alloc;

use alloc::{string::String, sync::Arc};
use kernel_hal::task::UserContext;
use object::task::Process;

pub fn syscall_handler(process: &Arc<Process>, user_ctx: &mut UserContext) {
    let [arg1, arg2, arg3, arg4, arg5, arg6] = user_ctx.get_syscall_args();
    let id = user_ctx.get_syscall_num();

    log::debug!(
        "syscall{}: {:#x} {:#x} {:#x} {:#x} {:#x} {:#x}",
        id,
        arg1,
        arg2,
        arg3,
        arg4,
        arg5,
        arg6
    );

    match id {
        0 => {
            let mut buf = alloc::vec![0u8; arg2 as usize];
            if let Err(error) = process.root_vmar().read(arg1, &mut buf) {
                log::error!("Failed to read msg: {}", error);
                user_ctx.set_syscall_ret(usize::MAX);
                return;
            };
            let Ok(msg) = String::from_utf8(buf) else {
                log::error!("Failed to parse msg");
                user_ctx.set_syscall_ret(usize::MAX);
                return;
            };
            log::info!("USER DEBUG: {}", msg);
        }
        _ => {}
    }
    user_ctx.set_syscall_ret(0);
}
