use core::arch::global_asm;

use loongarch64::{
    instructions::interrupt,
    registers::{ExceptionEntry, TimerConfigBuilder},
};

use crate::arch::task::{GeneralRegs, UserContext};

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
    log::error!("Trap: {:#x?}", f);
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
