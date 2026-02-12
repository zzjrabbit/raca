#[cfg(feature = "libos")]
#[doc(hidden)]
#[unsafe(link_section = ".rodata")]
#[unsafe(no_mangle)]
pub static SYSCALL_ENTRY: extern "C" fn() -> usize = empty;

#[cfg(feature = "libos")]
extern "C" fn empty() -> usize {
    0
}

#[cfg(feature = "libos")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn syscall() -> usize {
    SYSCALL_ENTRY()
}

#[cfg(target_arch = "x86_64")]
mod impl_ {
    #[macro_export]
    macro_rules! do_syscall {
        (@noret $index:expr $(,$arg:expr)*) => {
            do_syscall!(@impl $index, (), (in), (noreturn), $($arg),*)
        };

        ($index:expr $(,$arg:expr)*) => {{
            let ret;
            do_syscall!(@impl $index, (ret), (inlateout), (), $($arg),*);
            ret
        }};

        (@impl $index:expr, $ret:tt, $rax_mod:tt, $options:tt, $($arg:expr),*) => {
            crate::_syscall_impl! {
                [$index, $ret, $rax_mod, $options]
                [] ["rdi" "rsi" "rdx" "r10" "r8" "r9"] ($($arg),*)
            }
        };
    }

    #[macro_export]
    #[doc(hidden)]
    macro_rules! _syscall_impl {
        ([$index:expr, ($($ret:tt)?), ($rax_mod:tt), ($($options:tt)?)]
            [$($asm_args:tt)*] [$($_regs:tt)*] ()
        ) => {
            #[cfg(feature = "libos")]
            unsafe {
                core::arch::asm!(
                    "call {syscall_entry}",
                    syscall_entry = sym crate::regs::SYSCALL_ENTRY,
                    $($asm_args)*
                    $rax_mod("rax") $index as usize $(=> $ret)?,
                    clobber_abi("system"),
                    options(nostack $(, $options)?)
                )
            }
            #[cfg(not(feature = "libos"))]
            unsafe {
                core::arch::asm!(
                    "syscall",
                    $($asm_args)*
                    $rax_mod("rax") $index as usize $(=> $ret)?,
                    clobber_abi("system"),
                    options(nostack $(, $options)?)
                )
            }
        };

        ([$index:expr, $ret:tt, $rax_mod:tt, $options:tt]
            [$($asm_args:tt)*] [$reg:tt $($rest_regs:tt)*]
            ($arg:expr $(, $rest_args:expr)*)
        ) => {
            _syscall_impl! {
                [$index, $ret, $rax_mod, $options]
                [$($asm_args)* in($reg) $arg,] [$($rest_regs)*] ($($rest_args),*)
            }
        };

        ([$_index:expr, $_ret:tt, $_rax_mod:tt, $_noret:tt]
            [$($_asm_args:tt)*] [] ($arg:expr $(, $rest_args:expr)*)
        ) => {
            compile_error!(concat!("Syscall allows up to 6 arguments: ", stringify!($arg $(, $rest_args)*)));
        };
    }
}
