pub fn init() {
    inject_user_page_fault_handler(page_fault_handler);
}

use {
    crate::task::Process,
    kernel_hal::{platform::trap::CpuExceptionInfo, task::{PageFaultInfo, inject_user_page_fault_handler}},
};

pub(super) fn exception_handler(info: &CpuExceptionInfo) -> Result<(), ()> {
    if info.is_pf() {
        page_fault_handler(info)
    } else {
        Err(())
    }
}

pub(super) fn page_fault_handler(info: &CpuExceptionInfo) -> Result<(), ()> {
    let PageFaultInfo { addr, flags } = info.as_pf_info().ok_or(())?;

    let process = Process::current().unwrap();
    if process
        .root_vmar()
        .handle_page_fault(addr, flags)
        .map_err(|_| ())?
    {
        Ok(())
    } else {
        Err(())
    }
}
