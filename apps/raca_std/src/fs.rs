use syscall_macro::syscall;

#[repr(u8)]
pub enum OpenMode {
    Read = 0,
    Write = 1,
    ReadWrite = 2,
}

pub fn open(path: &str,mode:OpenMode) -> usize {
    const OPEN_SYSCALL_ID: u64 = 1;
    
    syscall!(OPEN_SYSCALL_ID, path.as_ptr() as usize, path.len(), mode as usize)
    
}

