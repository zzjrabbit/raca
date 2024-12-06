use syscall_macro::syscall;

use crate::Result;

#[repr(C, packed)]
#[derive(Clone, Copy, Debug)]
pub struct Process {
    binary_addr: usize,
    binary_len: usize,
    name_addr: usize,
    name_len: usize,
    stdin: usize,
    stdout: usize,
    cmd_line_addr: usize,
    cmd_line_len: usize,
}

impl Process {
    pub fn new(binary: &[u8], name: &str, stdin: usize, stdout: usize, cmd_line: &[u8]) -> Self {
        Self {
            binary_addr: binary.as_ptr() as usize,
            binary_len: binary.len(),
            name_addr: name.as_ptr() as usize,
            name_len: name.len(),
            stdin,
            stdout,
            cmd_line_addr: cmd_line.as_ptr() as usize,
            cmd_line_len: cmd_line.len(),
        }
    }

    pub fn run(&self) -> Result<usize> {
        const CREATE_PROCESS_SYSCALL_ID: u64 = 8;

        let ptr = self as *const _ as usize;
        syscall!(CREATE_PROCESS_SYSCALL_ID, ptr)
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Signal {
    pub ty: usize,
    pub data: [u64; 8],
}

pub fn has_signal(ty: usize) -> bool {
    const HAS_SIGNAL_SYSCALL_ID: u64 = 10;

    let result = syscall!(HAS_SIGNAL_SYSCALL_ID, ty);
    result.unwrap() == 1
}

pub fn start_wait_for_signal(ty: usize) -> Result<()> {
    const START_WAIT_FOR_SIGNAL_SYSCALL_ID: u64 = 13;

    syscall!(START_WAIT_FOR_SIGNAL_SYSCALL_ID, ty).map(|_| ())
}

pub fn get_signal(ty: usize, signal: &mut Signal) -> Result<()> {
    const GET_SIGNAL_SYSCALL_ID: u64 = 11;

    syscall!(GET_SIGNAL_SYSCALL_ID, ty, signal as *mut Signal as usize).map(|_| ())
}

pub fn done_signal(ty: usize) -> Result<()> {
    const DONE_SIGNAL_SYSCALL_ID: u64 = 12;

    syscall!(DONE_SIGNAL_SYSCALL_ID, ty).map(|_| ())
}

pub fn exit(code: usize) -> ! {
    const EXIT_SYSCALL_ID: u64 = 14;

    let _ = syscall!(EXIT_SYSCALL_ID, code);
    loop {}
}

/// wait for a process created by your APP
pub fn wait() -> Result<usize> {
    start_wait_for_signal(1)?;
    while !has_signal(1) {}
    let mut signal = Signal::default();
    get_signal(1, &mut signal).unwrap();
    done_signal(signal.ty)?;
    Ok(signal.data[0] as usize)
}
