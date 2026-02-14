use crate::{arch::trap::run_user, platform::task::kernel_task_entry};

#[derive(Debug, Clone)]
#[repr(C)]
pub struct TaskContext {
    sp: usize,
    fp: usize,
    s0: usize,
    s1: usize,
    s2: usize,
    s3: usize,
    s4: usize,
    s5: usize,
    s6: usize,
    s7: usize,
    s8: usize,
    ra: usize,
}

impl TaskContext {
    /// Creates a new `TaskContext`.
    pub(crate) const fn new() -> Self {
        Self {
            sp: 0,
            fp: 0,
            s0: 0,
            s1: 0,
            s2: 0,
            s3: 0,
            s4: 0,
            s5: 0,
            s6: 0,
            s7: 0,
            s8: 0,
            ra: 0,
        }
    }

    pub(crate) const fn set_ip(&mut self, ip: usize) {
        self.ra = ip;
    }

    pub(crate) const fn set_sp(&mut self, sp: usize) {
        self.sp = sp;
    }
}

#[unsafe(naked)]
pub extern "C" fn kernel_task_entry_wrapper() {
    core::arch::naked_asm!(
        ".cfi_startproc",
        ".cfi_undefined 1",
        "bl {kernel_task_entry}",
        ".cfi_endproc",
        kernel_task_entry = sym kernel_task_entry,
    );
}

macro_rules! first_context_switch_asm {
    () => {
        r#"ld.d  $sp, $a0, 0x0
        ld.d  $fp, $a0, 0x8
        ld.d  $s0, $a0, 0x10
        ld.d  $s1, $a0, 0x18
        ld.d  $s2, $a0, 0x20
        ld.d  $s3, $a0, 0x28
        ld.d  $s4, $a0, 0x30
        ld.d  $s5, $a0, 0x38
        ld.d  $s6, $a0, 0x40
        ld.d  $s7, $a0, 0x48
        ld.d  $s8, $a0, 0x50
        ld.d  $ra, $a0, 0x58
    
        ret"#
    };
}

#[unsafe(naked)]
pub unsafe extern "C" fn context_switch(nxt: *const TaskContext, cur: *mut TaskContext) {
    core::arch::naked_asm!(
        "st.d  $sp, $a1, 0x0",
        "st.d  $fp, $a1, 0x8",
        "st.d  $s0, $a1, 0x10",
        "st.d  $s1, $a1, 0x18",
        "st.d  $s2, $a1, 0x20",
        "st.d  $s3, $a1, 0x28",
        "st.d  $s4, $a1, 0x30",
        "st.d  $s5, $a1, 0x38",
        "st.d  $s6, $a1, 0x40",
        "st.d  $s7, $a1, 0x48",
        "st.d  $s8, $a1, 0x50",
        "st.d  $ra, $a1, 0x58",
        first_context_switch_asm!(),
    )
}

#[unsafe(naked)]
pub unsafe extern "C" fn first_context_switch(nxt: *const TaskContext) {
    core::arch::naked_asm!(first_context_switch_asm!())
}

#[repr(C)]
#[derive(Default)]
pub struct UserContext {
    /// General registers
    general: GeneralRegs,
    /// Pre-exception Mode Information
    prmd: usize,
    /// Exception Return Address
    era: usize,
    /// Extended Unit Enable
    euen: usize,
}

#[derive(Debug, Default, Clone, Copy)]
#[repr(C)]
pub struct GeneralRegs {
    pub zero: usize,
    pub ra: usize,
    pub tp: usize,
    pub sp: usize,
    pub a0: usize,
    pub a1: usize,
    pub a2: usize,
    pub a3: usize,
    pub a4: usize,
    pub a5: usize,
    pub a6: usize,
    pub a7: usize,
    pub t0: usize,
    pub t1: usize,
    pub t2: usize,
    pub t3: usize,
    pub t4: usize,
    pub t5: usize,
    pub t6: usize,
    pub t7: usize,
    pub t8: usize,
    pub r21: usize,
    pub fp: usize,
    pub s0: usize,
    pub s1: usize,
    pub s2: usize,
    pub s3: usize,
    pub s4: usize,
    pub s5: usize,
    pub s6: usize,
    pub s7: usize,
    pub s8: usize,
}

impl UserContext {
    pub fn enter_user_space(&mut self) {
        unsafe {
            run_user(self);
        }
    }
}

impl UserContext {
    /// Get number of syscall
    pub fn get_syscall_num(&self) -> usize {
        self.general.a7
    }

    /// Get return value of syscall
    pub fn get_syscall_ret(&self) -> usize {
        self.general.a0
    }

    /// Set return value of syscall
    pub fn set_syscall_ret(&mut self, ret: usize) {
        self.general.a0 = ret;
    }

    /// Get syscall args
    pub fn get_syscall_args(&self) -> [usize; 6] {
        [
            self.general.a0,
            self.general.a1,
            self.general.a2,
            self.general.a3,
            self.general.a4,
            self.general.a5,
        ]
    }

    /// Set instruction pointer
    pub fn set_ip(&mut self, ip: usize) {
        self.era = ip;
    }

    /// Set stack pointer
    pub fn set_sp(&mut self, sp: usize) {
        self.general.sp = sp;
    }

    /// Get stack pointer
    pub fn get_sp(&self) -> usize {
        self.general.sp
    }

    pub fn tls(&self) -> usize {
        self.general.tp
    }

    /// Set tls pointer
    pub fn set_tls(&mut self, tls: usize) {
        self.general.tp = tls;
    }

    pub fn set_first_arg(&mut self, arg: usize) {
        self.general.a0 = arg;
    }

    pub fn set_second_arg(&mut self, arg: usize) {
        self.general.a1 = arg;
    }
}
