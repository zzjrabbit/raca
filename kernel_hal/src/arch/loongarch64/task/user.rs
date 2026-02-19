use loongarch64::registers::{BadVirtAddr, ExceptionStatus};

use crate::{
    arch::trap::{CpuExceptionInfo, TrapFrame, handle_timer, run_user},
    mem::VirtAddr,
    task::ReturnReason,
};

#[repr(C)]
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

impl Default for UserContext {
    fn default() -> Self {
        Self {
            general: GeneralRegs::default(),
            prmd: 0b0111, // User mode, enable interrupt
            era: 0,
            euen: 0,
        }
    }
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
    pub fn enter_user_space(&mut self) -> ReturnReason {
        loop {
            unsafe {
                run_user(self);
            }
            let ecode = ExceptionStatus.read_ecode();
            match ecode {
                0 => {
                    handle_timer(&self.as_trap_frame());
                    break ReturnReason::KernelEvent;
                }
                0xb => {
                    self.era += 4;
                    break ReturnReason::Syscall;
                }
                _ => {
                    let badv = BadVirtAddr.read() as VirtAddr;
                    break ReturnReason::Exception(CpuExceptionInfo {
                        code: ecode as usize,
                        badv,
                    });
                }
            }
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

    fn as_trap_frame(&self) -> TrapFrame {
        TrapFrame {
            general: self.general,
            prmd: self.prmd,
            era: self.era,
            euen: self.euen,
        }
    }
}
