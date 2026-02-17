use alloc::{string::String, sync::Arc};
use errors::Errno;
use object::task::Process;

use crate::SyscallResult;

pub fn debug(process: &Arc<Process>, ptr: usize, len: usize) -> SyscallResult {
    let mut buf = alloc::vec![0u8; len];
    process.root_vmar().read(ptr, &mut buf)?;
    let Ok(msg) = String::from_utf8(buf) else {
        return Err(Errno::InvArg.no_message());
    };
    log::info!("USER DEBUG: {}", msg);
    Ok(0)
}
