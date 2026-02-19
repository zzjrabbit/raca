use crate::arch::trap::CpuExceptionInfo;

pub enum ReturnReason {
    Syscall,
    Exception(CpuExceptionInfo),
    KernelEvent,
}
