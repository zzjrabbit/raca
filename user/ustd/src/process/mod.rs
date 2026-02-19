use crate::syscall::sys_exit;

pub fn exit(exit_code: i32) -> ! {
    unsafe {
        sys_exit(exit_code).unwrap();
    }
    #[allow(clippy::empty_loop)]
    loop {}
}
