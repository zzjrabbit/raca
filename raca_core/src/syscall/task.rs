use core::slice::from_raw_parts;

use alloc::{boxed::Box, sync::Arc};
use spin::RwLock;

use crate::task::{signal::Signal, Process, SCHEDULER};

use crate::error::*;

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

pub fn create_process(info_addr: usize) -> Result<usize> {
    let info: &mut ProcessInfo = unsafe { &mut *(info_addr as *mut ProcessInfo) };

    let binary_addr = info.binary_addr;
    let binary_len = info.binary_len;
    let name_addr = info.name_addr;
    let name_len = info.name_len;
    let stdin = info.stdin;
    let stdout = info.stdout;

    let func = || {
        let buf = unsafe { from_raw_parts(binary_addr as *const u8, binary_len) };

        let name = unsafe { from_raw_parts(name_addr as *const u8, name_len) };

        let name = core::str::from_utf8(name);

        if let Err(_) = name {
            return Err(Error::InvalidUTF8String);
        }

        let current_process = get_current_process();

        let stdin_file = current_process
            .read()
            .file_descriptor_manager
            .file_descriptors
            .get(&stdin)
            .unwrap()
            .0
            .clone();
        let stdout_file = current_process
            .read()
            .file_descriptor_manager
            .file_descriptors
            .get(&stdout)
            .unwrap()
            .0
            .clone();

        let process = Process::new_user_process(name.unwrap(), buf, stdin_file, stdout_file);
        let pid = process.read().id;

        process.write().father = Some(Arc::downgrade(&get_current_process()));

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

pub fn has_signal(ty: usize) -> Result<usize> {
    let process = get_current_process();
    let process = process.read();
    if process.signal_manager.has_signal(ty) {
        return Ok(1);
    }
    Ok(0)
}

pub fn start_wait_for_signal(ty: usize) -> Result<usize> {
    let process = get_current_process();
    process.write().signal_manager.register_wait_for(ty);

    for thread in process.read().threads.iter() {
        SCHEDULER.lock().remove(Arc::downgrade(thread));
    }

    Ok(0)
}

pub fn get_signal(ty: usize, addr: usize) -> Result<usize> {
    let process = get_current_process();
    let mut process = process.write();

    if let Some(signal) = process.signal_manager.get_signal(ty) {
        let signal_ptr = addr as *mut Signal;
        unsafe {
            signal_ptr.write(signal);
        }
        Ok(0)
    } else {
        Err(Error::SignalNotFound)
    }
}

pub fn done_signal(ty: usize) -> Result<usize> {
    let process = get_current_process();
    process.write().signal_manager.delete_signal(ty);
    Ok(0)
}

pub fn exit(code: usize) -> Result<usize> {
    {
        let process = get_current_process();
        if let Some(ref father) = process.read().father {
            let father = father.upgrade().unwrap();
            let mut father = father.write();

            if father.signal_manager.register_signal(
                1,
                Signal {
                    ty: 1,
                    data: [code as u64, 0, 0, 0, 0, 0, 0, 0],
                },
            ) {
                for thread in father.threads.iter() {
                    SCHEDULER.lock().add(Arc::downgrade(thread));
                }
            }
        }
        process.read().exit_process();
    }
    Ok(0)
}
