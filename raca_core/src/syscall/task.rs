use core::slice::from_raw_parts;
use core::time::Duration;

use alloc::{boxed::Box, sync::Arc};
use spin::RwLock;
use x86_64::VirtAddr;

use crate::memory::ExtendedPageTable;
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

    let main_thread_stack = process.read().threads[0].read().context.rsp;

    let args_start = main_thread_stack - cmd_line.len();

    process
        .write()
        .page_table
        .write_to_mapped_address(&cmd_line, VirtAddr::new(args_start as u64));

    let argc = cmd_line.iter().filter(|x| **x == 0).count();

    let (father_env_start, father_env_len) = current_process.read().env_info;
    let env_data =
        unsafe { from_raw_parts(father_env_start as *const u8, father_env_len as usize) };

    let env_start = args_start - env_data.len();

    process
        .write()
        .page_table
        .write_to_mapped_address(env_data, VirtAddr::new(env_start as u64));

    process.read().threads[0].write().context.rsp = env_start;
    process.read().threads[0].write().context.rdi = args_start;
    process.read().threads[0].write().context.rsi = argc;
    process.read().threads[0].write().context.rdx = env_start;
    process.read().threads[0].write().context.rcx = env_data.len();

    process.write().env_info = (env_start, env_data.len());
    process.write().father = Some(Arc::downgrade(&get_current_process()));

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

    if let Some(thread) = SCHEDULER.lock().current_thread().upgrade() {
        thread.write().remove_after_schedule = true;
    }

    unsafe {
        core::arch::asm!("int 0x20");
    }
    Ok(0)
}

pub fn set_env(addr: usize, len: usize) -> Result<usize> {
    let process = get_current_process();

    process.write().env_info = (addr, len);

    Ok(0)
}
