use core::arch::naked_asm;

use bit_field::BitField;
use loongarch64::{
    VirtAddr,
    instructions::{interrupt, tlb},
    registers::{BadVirtAddr, ExceptionEntry, ExceptionStatus, TimerIntClear},
};

use crate::{
    arch::task::{GeneralRegs, UserContext},
    mem::{MMUFlags, USER_ASPACE_BASE, USER_ASPACE_SIZE},
    task::{PageFaultInfo, USER_PAGE_FAULT_HANDLER},
    timer::call_timer_callback_functions,
};

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

pub(super) fn handle_timer(trap_frame: &TrapFrame) {
    TimerIntClear.write(1);
    call_timer_callback_functions(trap_frame);
}

extern "C" fn trap_handler(f: &mut TrapFrame) {
    let estat = ExceptionStatus.read();
    let ecode = estat.get_bits(16..=21);
    let esubcode = estat.get_bits(22..=30);

    if ecode == 0 {
        if estat.get_bit(11) {
            handle_timer(f);
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

macro_rules! load_sp {
    ($r1: literal, $r2: literal) => {
        concat!("ld.d ", $r1, ", $sp, ", $r2, "*{XLENB}")
    };
}

macro_rules! store_sp {
    ($r1: literal, $r2: literal) => {
        concat!("st.d ", $r1, ", $sp, ", $r2, "*{XLENB}")
    };
}

const XLENB: usize = 8;
const SAVE_SCRATCH: usize = 0x37;
const LOONGARCH_CSR_PRMD: usize = 0x1;
const LOONGARCH_CSR_EUEN: usize = 0x2;
const LOONGARCH_CSR_ERA: usize = 0x6;

#[unsafe(naked)]
unsafe extern "C" fn trap_entry() {
    naked_asm!(
        // If coming from userspace, preserve the user stack pointer and load
        // the kernel stack pointer. If we came from the kernel, SAVE_SCRATCH
        // will contain 0, and we should continue on the current stack.
        "csrwr   $sp, {SAVE_SCRATCH}",
        "bnez    $sp, _trap_from_user",

        "_trap_from_kernel:",
        "csrrd   $sp, {SAVE_SCRATCH}",
        "addi.d  $sp, $sp, -35 * {XLENB}",

        "_trap_from_user:",
        // save general registers except $sp($r3)
        store_sp!("$r1", 1),
        store_sp!("$r2", 2),
        store_sp!("$r4", 4),
        store_sp!("$r5", 5),
        store_sp!("$r6", 6),
        store_sp!("$r7", 7),
        store_sp!("$r8", 8),
        store_sp!("$r9", 9),
        store_sp!("$r10", 10),
        store_sp!("$r11", 11),
        store_sp!("$r12", 12),
        store_sp!("$r13", 13),
        store_sp!("$r14", 14),
        store_sp!("$r15", 15),
        store_sp!("$r16", 16),
        store_sp!("$r17", 17),
        store_sp!("$r18", 18),
        store_sp!("$r19", 19),
        store_sp!("$r20", 20),
        store_sp!("$r21", 21),
        store_sp!("$r22", 22),
        store_sp!("$r23", 23),
        store_sp!("$r24", 24),
        store_sp!("$r25", 25),
        store_sp!("$r26", 26),
        store_sp!("$r27", 27),
        store_sp!("$r28", 28),
        store_sp!("$r29", 29),
        store_sp!("$r30", 30),
        store_sp!("$r31", 31),

        // save sp, prmd, era, euen
        "csrrd   $t0, {SAVE_SCRATCH}",
        "csrwr   $zero, {SAVE_SCRATCH}",     // SAVE_SCRATCH = 0 (kernel)
        "csrrd   $t1, {LOONGARCH_CSR_PRMD}",
        "csrrd   $t2, {LOONGARCH_CSR_ERA}",
        "csrrd   $t3, {LOONGARCH_CSR_EUEN}",
        store_sp!("$t0", 3),     // save sp
        store_sp!("$t1", 32),    // save prmd
        store_sp!("$t2", 33),    // save era
        store_sp!("$t3", 34),    // save euen

        "andi    $t1, $t1, 0x3",
        "bnez    $t1, _end_trap_from_user",

        "_end_trap_from_kernel:",
        "move    $a0, $sp",           // first arg is TrapFrame
        "la.local   $ra, _trap_return",
        "la.local   $t0, {trap_handler}",
        "jr      $t0",

        "_end_trap_from_user:",
        // load kernel-sp in UserContext.general.zero
        load_sp!("$sp", 0),
        // load callee-saved registers
        load_sp!("$s0", 0),
        load_sp!("$s1", 1),
        load_sp!("$s2", 2),
        load_sp!("$s3", 3),
        load_sp!("$s4", 4),
        load_sp!("$s5", 5),
        load_sp!("$s6", 6),
        load_sp!("$s7", 7),
        load_sp!("$s8", 8),
        load_sp!("$fp", 9),
        load_sp!("$ra", 10),
        load_sp!("$tp", 11),
        // not callee-saved, but is used to store cpu local storage
        load_sp!("$r21", 12),
        "addi.d  $sp, $sp, 13 * {XLENB}",
        "ret",
        XLENB = const XLENB,
        SAVE_SCRATCH = const SAVE_SCRATCH,
        LOONGARCH_CSR_PRMD = const LOONGARCH_CSR_PRMD,
        LOONGARCH_CSR_ERA = const LOONGARCH_CSR_ERA,
        LOONGARCH_CSR_EUEN = const LOONGARCH_CSR_EUEN,
        trap_handler = sym trap_handler,
    );
}

#[unsafe(naked)]
pub unsafe extern "C" fn run_user(ctx: &mut UserContext) {
    naked_asm!(
        // save callee-saved registers in kernel stack
        "addi.d  $sp, $sp, -13 * {XLENB}",
        store_sp!("$s0", 0),
        store_sp!("$s1", 1),
        store_sp!("$s2", 2),
        store_sp!("$s3", 3),
        store_sp!("$s4", 4),
        store_sp!("$s5", 5),
        store_sp!("$s6", 6),
        store_sp!("$s7", 7),
        store_sp!("$s8", 8),
        store_sp!("$fp", 9),
        store_sp!("$ra", 10),
        store_sp!("$tp", 11),
        // not callee-saved, but is used to store cpu local storage
        store_sp!("$r21", 12),


        "move    $t0, $sp",
        "move    $sp, $a0",
        store_sp!("$t0", 0),           // save kernel-sp in UserContext.general.zero
        "move    $t0, $sp",
        "csrwr   $t0, {SAVE_SCRATCH}", // SAVE_SCRATCH = bottom of TrapFrame/UserContext

        "_trap_return:",
        load_sp!("$t0", 32),         // t0 = prmd
        load_sp!("$t1", 33),         // t1 = era
        load_sp!("$t2", 34),         // t2 = euen

        "csrwr   $t0, {LOONGARCH_CSR_PRMD}",
        "csrwr   $t1, {LOONGARCH_CSR_ERA}",
        "csrwr   $t2, {LOONGARCH_CSR_EUEN}",

        // restore general registers except $sp($r3)
        load_sp!("$r1", 1),
        load_sp!("$r2", 2),
        load_sp!("$r4", 4),
        load_sp!("$r5", 5),
        load_sp!("$r6", 6),
        load_sp!("$r7", 7),
        load_sp!("$r8", 8),
        load_sp!("$r9", 9),
        load_sp!("$r10", 10),
        load_sp!("$r11", 11),
        load_sp!("$r12", 12),
        load_sp!("$r13", 13),
        load_sp!("$r14", 14),
        load_sp!("$r15", 15),
        load_sp!("$r16", 16),
        load_sp!("$r17", 17),
        load_sp!("$r18", 18),
        load_sp!("$r19", 19),
        load_sp!("$r20", 20),
        load_sp!("$r21", 21),
        load_sp!("$r22", 22),
        load_sp!("$r23", 23),
        load_sp!("$r24", 24),
        load_sp!("$r25", 25),
        load_sp!("$r26", 26),
        load_sp!("$r27", 27),
        load_sp!("$r28", 28),
        load_sp!("$r29", 29),
        load_sp!("$r30", 30),
        load_sp!("$r31", 31),
        // restore $sp last
        load_sp!("$r3", 3),

        // return from supervisor call
        "ertn",
        XLENB = const XLENB,
        SAVE_SCRATCH = const SAVE_SCRATCH,
        LOONGARCH_CSR_PRMD = const LOONGARCH_CSR_PRMD,
        LOONGARCH_CSR_ERA = const LOONGARCH_CSR_ERA,
        LOONGARCH_CSR_EUEN = const LOONGARCH_CSR_EUEN,
    );
}
