use core::slice::from_raw_parts;
use core::time::Duration;

use alloc::{boxed::Box, sync::Arc};
use spin::RwLock;

use crate::task::{Process, SCHEDULER, Thread, signal::Signal};

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
    cmd_line_addr: usize,
    cmd_line_len: usize,
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

pub fn create_thread(start: usize, stack_end: u64) -> Result<usize> {
    let current_process = get_current_process();

    let thread = Thread::new_user_thread_with_stack(
        Arc::downgrade(&current_process),
        start,
        x86_64::VirtAddr::new(stack_end),
    );
    let tid = thread.read().id;
    Ok(tid.0 as usize)
}

pub fn create_process(info_addr: usize) -> Result<usize> {
    let info: &mut ProcessInfo = unsafe { &mut *(info_addr as *mut ProcessInfo) };

    let binary_addr = info.binary_addr;
    let binary_len = info.binary_len;
    let name_addr = info.name_addr;
    let name_len = info.name_len;
    let stdin = info.stdin;
    let stdout = info.stdout;
    let cmd_line_addr = info.cmd_line_addr;
    let cmd_line_len = info.cmd_line_len;

    let buf = unsafe { from_raw_parts(binary_addr as *const u8, binary_len) };

    let name = unsafe { from_raw_parts(name_addr as *const u8, name_len) };

    let name = core::str::from_utf8(name);

    if let Err(_) = name {
        return Err(Error::InvalidUTF8String);
    }

    let cmd_line = unsafe { from_raw_parts(cmd_line_addr as *const u8, cmd_line_len) };
    let cmd_line = cmd_line.to_vec();

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

    let process = Process::new_user_process(name.unwrap(), buf, stdin_file, stdout_file)?;
    let pid = process.read().id;

    process.write().father = Some(Arc::downgrade(&get_current_process()));
    process.write().cmd_line = cmd_line;

    Ok(pid.0 as usize)
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
        thread.write().remove_after_schedule = true;
        SCHEDULER.lock().remove(Arc::downgrade(thread));
    }

    unsafe {
        core::arch::asm!("int 0x20");
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

            if father.signal_manager.register_signal(1, Signal {
                ty: 1,
                data: [code as u64, 0, 0, 0, 0, 0, 0, 0],
            }) {
                for thread in father.threads.iter() {
                    SCHEDULER.lock().add(Arc::downgrade(thread));
                }
            }
        }
        process.read().exit_process();
    }
    unsafe {
        core::arch::asm!("int 0x20");
    }
    Ok(0)
}

pub fn yield_thread() -> Result<usize> {
    unsafe {
        core::arch::asm!("int 0x20");
    }

    Ok(0)
}

pub fn sleep(ms: usize) -> Result<usize> {
    crate::task::timer::TIMER
        .lock()
        .add(Duration::from_micros(ms as u64));

    let scheduler = SCHEDULER.lock();
    let current_thread_weak = scheduler.current_thread();
    current_thread_weak
        .upgrade()
        .unwrap()
        .write()
        .remove_after_schedule = true;
    drop(scheduler);
    unsafe {
        core::arch::asm!("int 0x20");
    }
    Ok(0)
}
