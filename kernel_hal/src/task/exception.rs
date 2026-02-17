use spin::Once;

use crate::{arch::trap::CpuExceptionInfo, mem::{MMUFlags, VirtAddr}};

pub struct PageFaultInfo {
    pub addr: VirtAddr,
    pub flags: MMUFlags,
}

type PageFaultHandler = fn(&CpuExceptionInfo) -> core::result::Result<(), ()>;

pub(crate) static USER_PAGE_FAULT_HANDLER: Once<PageFaultHandler> = Once::new();

/// Injects a custom handler for page faults that occur in the kernel and
/// are caused by user-space address.
pub fn inject_user_page_fault_handler(handler: PageFaultHandler) {
    USER_PAGE_FAULT_HANDLER.call_once(|| handler);
}

