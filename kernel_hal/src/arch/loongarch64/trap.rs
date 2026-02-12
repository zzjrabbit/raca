use core::arch::global_asm;

use loongarch64::registers::ExceptionEntry;

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
extern "C" fn trap_handler(_f: &mut TrapFrame) {}

pub fn init() {
    ExceptionEntry.write(trap_handler as *const () as u64);
}
