use protocol::{
    FIRST_HANDLE, PROC_HANDLE_IDX, PROC_START_HANDLE_CNT, ProcessStartInfo, ReadBuffer,
    VMAR_HANDLE_IDX,
};

use crate::{
    ipc::Channel, os::raca::OwnedHandle, println, process::Process, syscall::sys_read_channel,
    vm::Vmar,
};

unsafe extern "Rust" {
    fn main(channel: &Channel) -> i32;
}

#[unsafe(no_mangle)]
extern "C" fn _start(info: *const ProcessStartInfo) -> ! {
    let ProcessStartInfo {
        vmar_base,
        vmar_size,
    } = unsafe { info.read() };
    println!(
        "vmar base {:#x} size {:#x} info addr {:p}",
        vmar_base, vmar_size, info
    );
    //println!("entered entry");

    let channel = FIRST_HANDLE;
    let mut handles = [0u32; PROC_START_HANDLE_CNT];
    unsafe {
        sys_read_channel(
            channel,
            &ReadBuffer::new_zero(),
            &ReadBuffer {
                addr: handles.as_mut_ptr() as usize,
                len: handles.len(),
                actual_len_addr: 0,
            },
        )
        .unwrap();
    }
    //println!("read channel");

    let process = handles[PROC_HANDLE_IDX];
    let vmar = handles[VMAR_HANDLE_IDX];

    let root_vmar =
        unsafe { Vmar::from_handle_base_size(OwnedHandle::from_raw(vmar), vmar_base, vmar_size) };
    super::heap::init(&root_vmar);
    let process = unsafe { Process::from_handle_vmar(OwnedHandle::from_raw(process), root_vmar) };
    crate::process::init(process);

    let channel = unsafe { Channel::from_handle(OwnedHandle::from_raw(channel)) };

    let exit_code = unsafe { main(&channel) };

    crate::process::exit(exit_code);
}
