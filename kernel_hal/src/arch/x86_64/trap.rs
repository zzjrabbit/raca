use crate::task::PageFaultInfo;

#[repr(C)]
#[derive(Debug, Default, Clone, Copy, Eq, PartialEq)]
pub struct TrapFrame {}

#[derive(Debug)]
pub struct CpuExceptionInfo;

impl CpuExceptionInfo {
    pub fn is_syscall(&self) -> bool {
        true
    }

    pub fn is_pf(&self) -> bool {
        false
    }

    pub fn as_pf_info(&self) -> Option<PageFaultInfo> {
        None
    }
}
