pub fn init() {
    #[cfg(not(feature = "libos"))]
    {
        use kernel_hal::platform::trap::inject_user_page_fault_handler;
        inject_user_page_fault_handler(page_fault_handler);
    }
}

#[cfg(not(feature = "libos"))]
use {
    crate::task::Process,
    kernel_hal::{mem::MMUFlags, platform::trap::CpuExceptionInfo},
};
#[cfg(not(feature = "libos"))]
pub(super) fn page_fault_handler(info: &CpuExceptionInfo) -> Result<(), ()> {
    let CpuExceptionInfo { code, badv } = info;
    let perm_required = match *code {
        0x1 => MMUFlags::READ,
        0x2 => MMUFlags::WRITE,
        0x3 => MMUFlags::EXECUTE,
        _ => return Err(()),
    };

    let process = Process::current().unwrap();
    if process
        .root_vmar()
        .handle_page_fault(*badv, perm_required)
        .map_err(|_| ())?
    {
        Ok(())
    } else {
        Err(())
    }
}

#[cfg(feature = "libos")]
pub(super) fn page_fault_handler(_info: ()) -> Result<(), ()> {
    Ok(())
}
