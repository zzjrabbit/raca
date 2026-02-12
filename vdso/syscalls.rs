gen_syscall! {
    fn sys_debug (0usize) (ptr: *const u8, len: usize);
    fn sys_empty (1usize) ();
}
