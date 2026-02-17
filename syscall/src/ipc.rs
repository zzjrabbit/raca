use alloc::sync::Arc;
use object::{
    ipc::Channel,
    object::{Handle, Rights},
    task::Process,
};

use crate::SyscallResult;

pub fn new_channel(
    process: &Arc<Process>,
    handle0_ptr: usize,
    handle1_ptr: usize,
) -> SyscallResult {
    let (channel0, channel1) = Channel::new();
    let handle0 = process.add_handle(Handle::new(channel0, Rights::ALL));
    let handle1 = process.add_handle(Handle::new(channel1, Rights::ALL));
    process.root_vmar().write_val(handle0_ptr, &handle0)?;
    process.root_vmar().write_val(handle1_ptr, &handle1)?;
    Ok(0)
}
