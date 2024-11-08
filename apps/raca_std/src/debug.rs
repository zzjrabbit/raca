use syscall_macro::syscall;

pub fn debug(message: &str) -> usize {
    const DEBUG_SYSCALL_ID: u64 = 0;
    
    syscall!(DEBUG_SYSCALL_ID, message.as_ptr() as usize, message.len())
    
}

