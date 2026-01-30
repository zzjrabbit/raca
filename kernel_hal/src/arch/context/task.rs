use core::arch::global_asm;

use crate::task::{TaskContextApi, kernel_task_entry};

global_asm!(include_str!("switch.asm"));

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

unsafe extern "C" {
    pub unsafe fn context_switch(nxt: *const TaskContext, cur: *mut TaskContext);
    pub unsafe fn first_context_switch(nxt: *const TaskContext);
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
