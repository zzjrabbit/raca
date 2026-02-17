use crate::arch::trap::CpuExceptionInfo;

pub enum ReturnReason {
    Syscall,
    Int(usize),
    Exception(CpuExceptionInfo),
}
