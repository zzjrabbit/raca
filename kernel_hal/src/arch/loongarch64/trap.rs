use core::arch::global_asm;

use bit_field::BitField;
use loongarch64::{
    VirtAddr,
    instructions::interrupt,
    registers::{BadVirtAddr, ExceptionEntry, ExceptionStatus, TimerConfigBuilder, TimerIntClear},
};

use crate::{
    arch::task::{GeneralRegs, UserContext},
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

    log::error!("Unhandled exception {:#x}!", ecode);
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
