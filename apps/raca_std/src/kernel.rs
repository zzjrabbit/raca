use syscall_macro::syscall;

pub fn insert_module(data: &[u8]) -> isize {
    let ptr = data.as_ptr() as usize;
    let size = data.len();

    const INSERT_MODULE_SYSCALL_ID: u64 = 9;

    syscall!(INSERT_MODULE_SYSCALL_ID,ptr,size)
}

