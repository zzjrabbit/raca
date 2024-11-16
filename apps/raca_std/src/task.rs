use syscall_macro::syscall;

#[repr(C, packed)]
#[derive(Clone, Copy, Debug)]
pub struct Process {
    binary_addr: usize,
    binary_len: usize,
    name_addr: usize,
    name_len: usize,
    stdin: usize,
    stdout: usize,
}

impl Process {
    pub fn new(binary: &'static [u8], name: &str, stdin: usize, stdout: usize) -> Self {
        Self {
            binary_addr: binary.as_ptr() as usize,
            binary_len: binary.len(),
            name_addr: name.as_ptr() as usize,
            name_len: name.len(),
            stdin,
            stdout,
        }
    }

    pub fn run(&self) {
        const CREATE_PROCESS_SYSCALL_ID: u64 = 8;

        let ptr = self as *const _ as usize;
        syscall!(CREATE_PROCESS_SYSCALL_ID,ptr);
    }
}
