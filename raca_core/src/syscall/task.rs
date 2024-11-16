use core::slice::from_raw_parts;

use alloc::{boxed::Box, sync::Arc};
use spin::RwLock;

use crate::{error::{RcError, RcResult}, task::{Process, SCHEDULER}};

#[repr(C, packed)]
#[derive(Clone, Copy, Debug)]
struct ProcessInfo {
    binary_addr: usize,
    binary_len: usize,
    name_addr: usize,
    name_len: usize,
    stdin: usize,
    stdout: usize,
}

fn get_current_process() -> Arc<RwLock<Box<Process>>> {
    SCHEDULER
        .lock()
        .current_thread()
        .upgrade()
        .unwrap()
        .read()
        .process
        .upgrade()
        .unwrap()
}

pub fn create_process(info_addr: usize) -> RcResult<usize> {
    let info: &mut ProcessInfo = unsafe {&mut *(info_addr as *mut ProcessInfo)};

    let binary_addr = info.binary_addr;
    let binary_len = info.binary_len;
    let name_addr = info.name_addr;
    let name_len = info.name_len;
    let stdin = info.stdin;
    let stdout = info.stdout;

    let func = || {
        let buf = unsafe {from_raw_parts(binary_addr as *const u8, binary_len)};

        let name = unsafe {from_raw_parts(name_addr as *const u8, name_len)};

        let name = core::str::from_utf8(name);

        if let Err(_) = name {
            return Err(RcError::INVALID_ARGS);
        }

        let current_process = get_current_process();

        let stdin_file = current_process.read().file_descriptor_manager.file_descriptors.get(&stdin).unwrap().0.clone();
        let stdout_file = current_process.read().file_descriptor_manager.file_descriptors.get(&stdout).unwrap().0.clone();

        let process = Process::new_user_process(name.unwrap(), buf,stdin_file, stdout_file);
        let pid = process.read().id;

        /*if let Some(stdin) = get_inode_by_fd(stdin) {
            let stdout = get_inode_by_fd(stdout).unwrap();
            init_file_descriptor_manager_with_stdin_stdout(pid, stdin, stdout);
        } else {
            init_file_descriptor_manager(pid);
        }

        process.write().father = Some(Arc::downgrade(&get_current_process()));*/

        Ok(pid.0 as usize)
    };

    func()

    //x86_64::instructions::interrupts::without_interrupts(func)
}
