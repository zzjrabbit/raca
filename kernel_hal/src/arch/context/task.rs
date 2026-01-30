use crate::task::{TaskContextApi, kernel_task_entry};

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
pub extern "C" fn context_switch(nxt: *const TaskContext, cur: *mut TaskContext) {
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
pub extern "C" fn first_context_switch(nxt: *const TaskContext) {
    core::arch::naked_asm!(first_context_switch_asm!())
}

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
    pub const fn new() -> Self {
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
}

impl TaskContextApi for TaskContext {
    unsafe fn set_program_counter(&mut self, pc: usize) {
        self.ra = pc;
    }

    unsafe fn set_stack_pointer(&mut self, sp: usize) {
        self.sp = sp;
    }
}
