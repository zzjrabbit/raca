use alloc::sync::Arc;
use object::{
    object::{Handle, Rights},
    task::{HandleId, Process, Thread},
};
use protocol::FIRST_HANDLE;

use crate::{SyscallResult, syscall_handler};

pub fn new_process(
    process: &Arc<Process>,
    handle_ptr: usize,
    vmar_handle_ptr: usize,
    base_ptr: usize,
    size_ptr: usize,
) -> SyscallResult {
    let vmar = process.root_vmar();

    let child = Process::new();
    let child_vmar = child.root_vmar();

    let handle = process.add_handle(Handle::new(child.clone(), Rights::PROCESS));
    vmar.write_val(handle_ptr, &handle)?;

    let vmar_handle = process.add_handle(Handle::new(child_vmar.clone(), Rights::VMAR));
    vmar.write_val(vmar_handle_ptr, &vmar_handle)?;

    vmar.write_val(base_ptr, &child_vmar.base())?;
    vmar.write_val(size_ptr, &child_vmar.size())?;

    Ok(0)
}

pub fn kill_process(process: &Arc<Process>, handle: u32) -> SyscallResult {
    let child =
        process.find_object_with_rights::<Process>(HandleId::from_raw(handle), Rights::MANAGE)?;
    child.kill();
    Ok(0)
}

pub fn exit(process: &Arc<Process>, exit_code: i32) -> SyscallResult {
    log::info!("Process {} exited with code {}.", process.id(), exit_code);
    process.exit(exit_code);
    Ok(0)
}

pub fn start_process(
    process: &Arc<Process>,
    handle: u32,
    thread_handle: u32,
    boot_handle: u32,
    entry: usize,
    stack: usize,
    start_info_addr: usize,
) -> SyscallResult {
    let child =
        process.find_object_with_rights::<Process>(HandleId::from_raw(handle), Rights::MANAGE)?;
    let thread = process
        .find_object_with_rights::<Thread>(HandleId::from_raw(thread_handle), Rights::MANAGE)?;

    let boot_handle = process.remove_handle(HandleId::from_raw(boot_handle))?;
    let boot_handle = child.add_handle(boot_handle);
    assert_eq!(boot_handle.as_raw(), FIRST_HANDLE);

    child.start(
        thread,
        entry,
        stack,
        |ctx| {
            ctx.set_first_arg(start_info_addr);
        },
        syscall_handler,
    );
    Ok(0)
}

pub fn new_thread(process: &Arc<Process>, proc_handle: u32, handle_ptr: usize) -> SyscallResult {
    let child = process
        .find_object_with_rights::<Process>(HandleId::from_raw(proc_handle), Rights::MANAGE)?;
    let thread = child.new_thread();
    let handle = process.add_handle(Handle::new(thread, Rights::THREAD));
    process.root_vmar().write_val(handle_ptr, &handle)?;
    Ok(0)
}

pub fn start_thread(
    process: &Arc<Process>,
    handle: u32,
    entry: usize,
    stack: usize,
    first_arg: usize,
) -> SyscallResult {
    let thread =
        process.find_object_with_rights::<Thread>(HandleId::from_raw(handle), Rights::MANAGE)?;
    thread.start_user(
        process.clone(),
        entry,
        stack,
        |ctx| {
            ctx.set_first_arg(first_arg);
        },
        syscall_handler,
    );
    Ok(0)
}

pub fn exit_thread() -> SyscallResult {
    let current_thread = Thread::current().unwrap();
    current_thread.exit();
    Ok(0)
}

pub fn kill_thread(process: &Arc<Process>, handle: u32) -> SyscallResult {
    let thread =
        process.find_object_with_rights::<Thread>(HandleId::from_raw(handle), Rights::MANAGE)?;
    thread.kill();
    Ok(0)
}
