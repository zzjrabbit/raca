use protocol::ProcessStartInfo;

use crate::{ipc::Channel, os::raca::OwnedHandle, process::Process, vm::Vmar};

unsafe extern "Rust" {
    fn main(process: Process, channel: Channel) -> i32;
}

#[unsafe(no_mangle)]
extern "C" fn _start(info: *const ProcessStartInfo) -> ! {
    let ProcessStartInfo {
        process,
        _reserved: _,
        channel,
        vmar,
        vmar_base,
        vmar_size,
    } = unsafe { info.read() };
    let root_vmar =
        unsafe { Vmar::from_handle_base_size(OwnedHandle::from_raw(vmar), vmar_base, vmar_size) };
    super::heap::init(&root_vmar);
    let process = unsafe { Process::from_handle_vmar(OwnedHandle::from_raw(process), root_vmar) };

    let channel = unsafe { Channel::from_handle(OwnedHandle::from_raw(channel)) };

    let exit_code = unsafe { main(process, channel) };

    crate::process::exit(exit_code);
}
