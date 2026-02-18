use alloc::sync::Arc;
use object::task::{HandleId, Process};

use crate::SyscallResult;

pub fn remove_handle(process: &Arc<Process>, handle: u32) -> SyscallResult {
    process.remove_handle(HandleId::from_raw(handle))?;
    Ok(0)
}
