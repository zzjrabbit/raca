#![no_std]

use kernel_hal::arch::task::UserContext;

pub fn syscall_handler(_user_ctx: &mut UserContext) {}
