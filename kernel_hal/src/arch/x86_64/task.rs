use std::arch::naked_asm;

/// User space context
#[derive(Debug, Default, Clone, Copy, Eq, PartialEq)]
#[repr(C)]
pub struct UserContext {
    pub general: GeneralRegs,
    pub trap_num: usize,
    pub error_code: usize,
}

/// General registers
#[derive(Debug, Default, Clone, Copy, Eq, PartialEq)]
#[repr(C)]
pub struct GeneralRegs {
    pub rax: usize,
    pub rbx: usize,
    pub rcx: usize,
    pub rdx: usize,
    pub rsi: usize,
    pub rdi: usize,
    pub rbp: usize,
    pub rsp: usize,
    pub r8: usize,
    pub r9: usize,
    pub r10: usize,
    pub r11: usize,
    pub r12: usize,
    pub r13: usize,
    pub r14: usize,
    pub r15: usize,
    pub rip: usize,
    pub rflags: usize,
    pub fsbase: usize,
    pub gsbase: usize,
}

impl UserContext {
    /// Get number of syscall
    pub fn get_syscall_num(&self) -> usize {
        self.general.rax
    }

    /// Get return value of syscall
    pub fn get_syscall_ret(&self) -> usize {
        self.general.rax
    }

    /// Set return value of syscall
    pub fn set_syscall_ret(&mut self, ret: usize) {
        self.general.rax = ret;
    }

    /// Get syscall args
    pub fn get_syscall_args(&self) -> [usize; 6] {
        [
            self.general.rdi,
            self.general.rsi,
            self.general.rdx,
            self.general.r10,
            self.general.r8,
            self.general.r9,
        ]
    }

    /// Set instruction pointer
    pub fn set_ip(&mut self, ip: usize) {
        self.general.rip = ip;
    }

    /// Set stack pointer
    pub fn set_sp(&mut self, sp: usize) {
        self.general.rsp = sp;
    }

    /// Get stack pointer
    pub fn get_sp(&self) -> usize {
        self.general.rsp
    }

    /// Set tls pointer
    pub fn set_tls(&mut self, tls: usize) {
        self.general.fsbase = tls;
    }

    pub fn set_first_arg(&mut self, arg: usize) {
        self.general.rdi = arg;
    }
}

impl UserContext {
    /// Go to user context by function return, within the same privilege level.
    ///
    /// User program should call `syscall_fn_entry()` to return back.
    /// Trap reason and error code will always be set to 0x100 and 0.
    #[cfg(feature = "libos")]
    pub fn run_fncall(&mut self) {
        unsafe {
            syscall_fn_return(self);
        }
        self.trap_num = 0x100;
        self.error_code = 0;
    }

    pub fn enter_user_space(&mut self) {
        #[cfg(feature = "libos")]
        self.run_fncall();
        #[cfg(not(feature = "libos"))]
        unimplemented!()
    }
}

macro_rules! switch_to_kernel_stack {
    () => {
        r#"
            mov rsp, fs:48          # rsp = kernel fsbase
            mov rsp, [rsp + 64]     # rsp = kernel stack
        "#
    };
}

macro_rules! save_kernel_stack {
    () => {
        r#"
            mov fs:64, rsp
        "#
    };
}

macro_rules! push_user_fsbase {
    () => {
        r#"
            push fs:0
        "#
    };
}

macro_rules! switch_to_kernel_fsbase {
    () => {
        r#"
            mov eax, 158            # SYS_arch_prctl
            mov edi, 0x1002         # SET_FS
            mov rsi, fs:48          # rsi = kernel fsbase
            syscall
        "#
    };
}

macro_rules! pop_user_fsbase {
    () => {
        r#"
            mov rsi, [rsp + 18 * 8] # rsi = user fsbase
            mov rdx, fs:0           # rdx = kernel fsbase
            test rsi, rsi
            jnz 1f                  # if not 0, goto set
            0:  lea rsi, [rdx + 72]     # rsi = init user fsbase
            mov [rsi], rsi          # user_fs:0 = user fsbase
            1:  mov eax, 158            # SYS_arch_prctl
            mov edi, 0x1002         # SET_FS
            syscall                 # set fsbase
            mov fs:48, rdx          # user_fs:48 = kernel fsbase
        "#
    };
}

#[cfg(feature = "libos")]
#[unsafe(naked)]
pub unsafe extern "sysv64" fn syscall_fn_entry() {
    naked_asm!(
        r#"
        # save rsp
        lea r11, [rsp + 8]      # save rsp to r11 (clobber)
        "#,
        switch_to_kernel_stack!(),
        r#"
        pop rsp
        lea rsp, [rsp + 20*8]   # rsp = top of trap frame

        # push trap frame (struct GeneralRegs)
        push 0                  # ignore gs_base
        "#,
        push_user_fsbase!(),
        r#"
        pushfq                  # push rflags
        push [r11 - 8]          # push rip
        push r15
        push r14
        push r13
        push r12
        push r11
        push r10
        push r9
        push r8
        push r11                # push rsp
        push rbp
        push rdi
        push rsi
        push rdx
        push rcx
        push rbx
        push rax

        # restore callee-saved registers
        "#,
        switch_to_kernel_stack!(),
        r#"
        pop rbx
        pop rbx
        pop rbp
        pop r12
        pop r13
        pop r14
        pop r15
        "#,
        switch_to_kernel_fsbase!(),
        r#"
        # go back to Rust
        ret
        "#
    );
}

#[cfg(feature = "libos")]
#[unsafe(naked)]
unsafe extern "sysv64" fn syscall_fn_return(ctx: &mut UserContext) {
    naked_asm!(
        r#"
        # save callee-saved registers
        push r15
        push r14
        push r13
        push r12
        push rbp
        push rbx

        push rdi
        "#,
        save_kernel_stack!(),
        r#"
        mov rsp, rdi
        "#,
        pop_user_fsbase!(),
        r#"
        # pop trap frame (struct GeneralRegs)
        pop rax
        pop rbx
        pop rcx
        pop rdx
        pop rsi
        pop rdi
        pop rbp
        pop r8                  # skip rsp
        pop r8
        pop r9
        pop r10
        pop r11
        pop r12
        pop r13
        pop r14
        pop r15
        pop r11                 # r11 = rip. FIXME: don't overwrite r11!
        popfq                   # pop rflags
        mov rsp, [rsp - 8*11]   # restore rsp
        jmp r11                 # restore rip
        "#
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::arch::global_asm;

    // Mock user program to dump registers at stack.
    global_asm!(
        r#"
        dump_registers:
        push r15
        push r14
        push r13
        push r12
        push r11
        push r10
        push r9
        push r8
        push rsp
        push rbp
        push rdi
        push rsi
        push rdx
        push rcx
        push rbx
        push rax
        
        add rax, 10
        add rbx, 10
        add rcx, 10
        add rdx, 10
        add rsi, 10
        add rdi, 10
        add rbp, 10
        add r8, 10
        add r9, 10
        add r10, 10
        add r11, 10
        add r12, 10
        add r13, 10
        add r14, 10
        add r15, 10
        
        call {syscall_fn_entry}
        "#, 
        syscall_fn_entry = sym syscall_fn_entry,
    );

    #[test]
    fn run_fncall() {
        unsafe extern "sysv64" {
            fn dump_registers();
        }
        let mut stack = [0u8; 0x1000];
        let mut cx = UserContext {
            general: GeneralRegs {
                rax: 0,
                rbx: 1,
                rcx: 2,
                rdx: 3,
                rsi: 4,
                rdi: 5,
                rbp: 6,
                rsp: stack.as_mut_ptr() as usize + 0x1000,
                r8: 8,
                r9: 9,
                r10: 10,
                r11: 11,
                r12: 12,
                r13: 13,
                r14: 14,
                r15: 15,
                rip: dump_registers as *const () as usize,
                rflags: 0,
                fsbase: 0, // don't set to non-zero garbage value
                gsbase: 0,
            },
            trap_num: 0,
            error_code: 0,
        };
        cx.run_fncall();
        // check restored registers
        let general = unsafe { *(cx.general.rsp as *const GeneralRegs) };
        assert_eq!(
            general,
            GeneralRegs {
                rax: 0,
                rbx: 1,
                rcx: 2,
                rdx: 3,
                rsi: 4,
                rdi: 5,
                rbp: 6,
                // skip rsp
                r8: 8,
                r9: 9,
                r10: 10,
                // skip r11
                r12: 12,
                r13: 13,
                r14: 14,
                r15: 15,
                ..general
            }
        );
        // check saved registers
        assert_eq!(
            cx.general,
            GeneralRegs {
                rax: 10,
                rbx: 11,
                rcx: 12,
                rdx: 13,
                rsi: 14,
                rdi: 15,
                rbp: 16,
                // skip rsp
                r8: 18,
                r9: 19,
                r10: 20,
                // skip r11
                r12: 22,
                r13: 23,
                r14: 24,
                r15: 25,
                ..cx.general
            }
        );
        assert_eq!(cx.trap_num, 0x100);
        assert_eq!(cx.error_code, 0);
    }
}
