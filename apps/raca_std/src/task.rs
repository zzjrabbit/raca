use syscall_macro::{syscall, syscall_noret};

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
    env_addr: usize,
    env_len: usize,
}

impl Process {
    pub fn new(binary: &[u8], name: &str, stdin: usize, stdout: usize, cmd_line: &[u8], env_addr: usize, env_len: usize) -> Self {
        Self {
            binary_addr: binary.as_ptr() as usize,
            binary_len: binary.len(),
            name_addr: name.as_ptr() as usize,
            name_len: name.len(),
            stdin,
            stdout,
            cmd_line_addr: cmd_line.as_ptr() as usize,
            cmd_line_len: cmd_line.len(),
            env_len,
            env_addr,
        }
    }

    pub fn run(&self) -> Result<usize> {
        const CREATE_PROCESS_SYSCALL_ID: u64 = 8;
        
        syscall!(CREATE_PROCESS_SYSCALL_ID, fn create_process(ptr: usize) -> Result<usize>);

        let ptr = self as *const _ as usize;
        create_process(ptr)
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Signal {
    pub ty: usize,
    pub data: [u64; 8],
}

pub fn has_signal(ty: usize) -> bool {
    const HAS_SIGNAL_SYSCALL_ID: u64 = 10;
    syscall!(HAS_SIGNAL_SYSCALL_ID, pub fn has_signal(ty: usize) -> Result<usize>);
    let Ok(code) = has_signal(ty) else {
        return false;
    };
    code == 1
}

const START_WAIT_FOR_SIGNAL_SYSCALL_ID: u64 = 13;
syscall!(START_WAIT_FOR_SIGNAL_SYSCALL_ID, pub fn start_wait_for_signal(ty: usize) -> Result<()>);

pub fn get_signal(ty: usize, signal: &mut Signal) -> Result<()> {
    const GET_SIGNAL_SYSCALL_ID: u64 = 11;
    syscall!(GET_SIGNAL_SYSCALL_ID, fn get_signal(ty: usize, signal: *mut Signal) -> Result<()>);

    get_signal(ty, signal)
}

const DONE_SIGNAL_SYSCALL_ID: u64 = 12;
syscall!(DONE_SIGNAL_SYSCALL_ID, pub fn done_signal(ty: usize) -> Result<()>);


const EXIT_SYSCALL_ID: u64 = 14;
syscall_noret!(EXIT_SYSCALL_ID, pub fn exit(code: usize) -> !);

/// wait for a process created by your APP
pub fn wait() -> Result<usize> {
    start_wait_for_signal(1)?;
    while !has_signal(1) {}
    let mut signal = Signal::default();
    get_signal(1, &mut signal).unwrap();
    done_signal(signal.ty)?;
    Ok(signal.data[0] as usize)
}
