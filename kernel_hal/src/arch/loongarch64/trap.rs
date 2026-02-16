use core::arch::global_asm;

use bit_field::BitField;
use loongarch64::{
    VirtAddr,
    instructions::{interrupt, tlb},
    registers::{BadVirtAddr, ExceptionEntry, ExceptionStatus, TimerConfigBuilder, TimerIntClear},
};
use spin::Once;

use crate::{
    arch::task::{GeneralRegs, UserContext},
    mem::{USER_ASPACE_BASE, USER_ASPACE_SIZE},
    timer::call_timer_callback_functions,
};

global_asm!(include_str!("trap.S"));

unsafe extern "C" {
    pub unsafe fn trap_entry();
    pub unsafe fn run_user(regs: &mut UserContext);
}

#[derive(Debug, Default, Clone, Copy)]
#[repr(C)]
pub struct TrapFrame {
    /// General registers
    pub general: GeneralRegs,
    /// Pre-exception Mode Information
    pub prmd: usize,
    /// Exception Return Address
    pub era: usize,
    /// Extended Unit Enable
    pub euen: usize,
}

#[unsafe(no_mangle)]
extern "C" fn trap_handler(f: &mut TrapFrame) {
    let estat = ExceptionStatus.read();
    let ecode = estat.get_bits(16..=21);
    let esubcode = estat.get_bits(22..=30);

    if ecode == 0 {
        if estat.get_bit(11) {
            call_timer_callback_functions(f);
            TimerIntClear.write(1);
        } else {
            log::warn!("Unknown interrupt!");
        }
        return;
    }

    let badv = VirtAddr::new(BadVirtAddr.read());

    if matches!(ecode, 0x1..0x8) {
        tlb::flush(badv);
        if let Some(handler) = USER_PAGE_FAULT_HANDLER.get()
            && (USER_ASPACE_BASE..(USER_ASPACE_BASE + USER_ASPACE_SIZE))
                .contains(&(badv.as_u64() as usize))
            && handler(&CpuExceptionInfo {
                code: ecode as usize,
                badv: badv.as_u64() as crate::mem::VirtAddr,
            })
            .is_ok()
        {
            log::debug!("page fault handled");
            return;
        }
    }

    log::error!("Unhandled exception {:#x} {:#x}!", ecode, esubcode);
    log::error!("BADV: {:#x}", badv);
    log::error!("Trap Frame: {:#x?}", f);

    panic!("Unrecoverable Exception");
}

pub fn init() {
    TimerConfigBuilder::new()
        .initial_value(1000000 >> 2)
        .set_enabled(true)
        .set_periodic(true)
        .done();
    ExceptionEntry.write(trap_entry as *const () as u64);
}

pub fn enable_int() {
    interrupt::enable();
}

pub fn disable_int() {
    interrupt::disable();
}

static USER_PAGE_FAULT_HANDLER: Once<fn(&CpuExceptionInfo) -> core::result::Result<(), ()>> =
    Once::new();

#[derive(Debug)]
pub struct CpuExceptionInfo {
    pub code: usize,
    pub badv: crate::mem::VirtAddr,
}

/// Injects a custom handler for page faults that occur in the kernel and
/// are caused by user-space address.
pub fn inject_user_page_fault_handler(
    handler: fn(info: &CpuExceptionInfo) -> core::result::Result<(), ()>,
) {
    USER_PAGE_FAULT_HANDLER.call_once(|| handler);
}
