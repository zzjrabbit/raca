use alloc::sync::Arc;
use object::task::{HandleId, Process};

use crate::SyscallResult;

pub fn remove_handle(process: &Arc<Process>, handle: u32) -> SyscallResult {
    process.remove_handle(HandleId::from_raw(handle))?;
    Ok(0)
}

pub fn duplicate_handle(
    process: &Arc<Process>,
    handle: u32,
    new_handle_ptr: usize,
) -> SyscallResult {
    let handle = process.get_handle(HandleId::from_raw(handle))?;

    let new_handle = process.add_handle(handle);
    process.root_vmar().write_val(new_handle_ptr, &new_handle)?;

    Ok(0)
}
