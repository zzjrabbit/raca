use alloc::{sync::Arc, vec};
use errors::Errno;
use object::{
    ipc::{Channel, MessagePacket},
    object::{Handle, Rights},
    task::{HandleId, Process},
};
use protocol::{ReadBuffer, WriteBuffer};

use crate::SyscallResult;

pub fn new_channel(
    process: &Arc<Process>,
    handle0_ptr: usize,
    handle1_ptr: usize,
) -> SyscallResult {
    let (channel0, channel1) = Channel::new();
    let handle0 = process.add_handle(Handle::new(channel0, Rights::ALL));
    let handle1 = process.add_handle(Handle::new(channel1, Rights::ALL));
    process.root_vmar().write_val(handle0_ptr, &handle0)?;
    process.root_vmar().write_val(handle1_ptr, &handle1)?;
    Ok(0)
}

pub fn read_channel(
    process: &Arc<Process>,
    handle: u32,
    data_buffer: usize,
    handle_buffer: usize,
) -> SyscallResult {
    let vmar = process.root_vmar();

    let data_buffer: ReadBuffer = vmar.read_val(data_buffer)?;
    let handle_buffer: ReadBuffer = vmar.read_val(handle_buffer)?;

    let channel =
        process.find_object_with_rights::<Channel>(HandleId::from_raw(handle), Rights::READ)?;
    let mut msg = channel.read()?;

    let actual_data_len = msg.data.len();
    let actual_handle_len = msg.handles.len();

    if data_buffer.actual_len_addr != 0 {
        process
            .root_vmar()
            .write_val(data_buffer.actual_len_addr, &actual_data_len)?;
    }
    if handle_buffer.actual_len_addr != 0 {
        process
            .root_vmar()
            .write_val(handle_buffer.actual_len_addr, &actual_handle_len)?;
    }

    if actual_data_len > data_buffer.len || actual_handle_len > handle_buffer.len {
        return Err(Errno::TooBig.no_message());
    }

    process.root_vmar().write(data_buffer.addr, &msg.data)?;

    for (id, handle) in msg.handles.drain(..).enumerate() {
        let handle = process.add_handle(handle);
        process
            .root_vmar()
            .write_val(handle_buffer.addr + id * size_of::<HandleId>(), &handle)?;
    }

    Ok(0)
}

pub fn write_channel(
    process: &Arc<Process>,
    handle: u32,
    data_buffer_addr: usize,
    handle_buffer_addr: usize,
) -> SyscallResult {
    let vmar = process.root_vmar();

    let data_buffer: WriteBuffer = vmar.read_val(data_buffer_addr)?;
    let handle_buffer: WriteBuffer = vmar.read_val(handle_buffer_addr)?;

    let channel =
        process.find_object_with_rights::<Channel>(HandleId::from_raw(handle), Rights::WRITE)?;

    let mut data = vec![0u8; data_buffer.len];
    vmar.read(data_buffer.addr, &mut data)?;
    let handles =
        vmar.read_array_map::<HandleId, _>(handle_buffer.addr, handle_buffer.len, |h| {
            process.remove_handle_with_rights(h, Rights::TRANSFER)
        })?;

    let msg = MessagePacket { data, handles };
    channel.write(msg)?;

    Ok(0)
}
