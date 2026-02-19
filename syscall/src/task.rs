use alloc::sync::Arc;
use object::task::Process;

use crate::SyscallResult;

pub fn exit(process: &Arc<Process>, exit_code: i32) -> SyscallResult {
    process.exit(exit_code);
    Ok(0)
}
