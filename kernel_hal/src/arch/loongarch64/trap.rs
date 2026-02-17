use core::arch::global_asm;

use bit_field::BitField;
use loongarch64::{
    VirtAddr,
    instructions::{interrupt, tlb},
    registers::{BadVirtAddr, ExceptionEntry, ExceptionStatus, TimerConfigBuilder, TimerIntClear},
};

use crate::{
    arch::task::{GeneralRegs, UserContext},
    mem::{MMUFlags, USER_ASPACE_BASE, USER_ASPACE_SIZE},
    task::{PageFaultInfo, USER_PAGE_FAULT_HANDLER},
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

#[derive(Debug)]
pub struct CpuExceptionInfo {
    pub code: usize,
    pub badv: crate::mem::VirtAddr,
}

impl CpuExceptionInfo {
    pub fn is_pf(&self) -> bool {
        matches!(self.code, 1..=8)
    }

    pub fn is_syscall(&self) -> bool {
        matches!(self.code, 0xb)
    }

    pub fn as_pf_info(&self) -> Option<PageFaultInfo> {
        matches!(self.code, 1..=3).then_some(PageFaultInfo {
            addr: self.badv,
            flags: match self.code {
                1 => MMUFlags::READ,
                2 => MMUFlags::WRITE,
                3 => MMUFlags::EXECUTE,
                _ => unreachable!(),
            },
        })
    }
}
