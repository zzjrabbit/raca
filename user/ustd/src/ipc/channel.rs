use core::{alloc::Layout, ptr::addr_of_mut, slice::from_raw_parts_mut};

use alloc::{
    alloc::{alloc, dealloc},
    vec::Vec,
};
use errors::Result;
use protocol::{ReadBuffer, WriteBuffer};

use crate::{
    os::raca::OwnedHandle,
    syscall::{sys_new_channel, sys_read_channel, sys_write_channel},
};

pub struct Channel(pub(crate) OwnedHandle);

#[derive(Default)]
pub struct MessagePacket {
    pub data: Vec<u8>,
    pub handles: Vec<OwnedHandle>,
}

impl Channel {
    pub fn new() -> Result<(Self, Self)> {
        let mut raw_handle0 = 0;
        let mut raw_handle1 = 0;
        unsafe {
            sys_new_channel(&mut raw_handle0, &mut raw_handle1)?;
            Ok((
                Self::from_handle(OwnedHandle::from_raw(raw_handle0)),
                Self::from_handle(OwnedHandle::from_raw(raw_handle1)),
            ))
        }
    }

    pub unsafe fn from_handle(handle: OwnedHandle) -> Self {
        Self(handle)
    }
}

impl Channel {
    pub fn read(&self) -> Result<MessagePacket> {
        const MAX_DATA_LEN: usize = 64 * 1024 * 1024;
        const MAX_HANDLE_COUNT: usize = 16 * 1024 * 1024;

        let data_layout = Layout::from_size_align(MAX_DATA_LEN, 1).unwrap();
        let raw_data = unsafe { from_raw_parts_mut(alloc(data_layout), MAX_DATA_LEN) };

        let raw_handles_layout =
            Layout::from_size_align(MAX_HANDLE_COUNT * size_of::<u32>(), align_of::<u32>())
                .unwrap();
        let raw_handles =
            unsafe { from_raw_parts_mut(alloc(raw_handles_layout) as *mut u32, MAX_HANDLE_COUNT) };

        let mut data_len = 0usize;
        let mut handle_count = 0usize;
        unsafe {
            sys_read_channel(
                self.0.as_raw(),
                &ReadBuffer {
                    addr: raw_data.as_mut_ptr() as usize,
                    len: raw_data.len(),
                    actual_len_addr: addr_of_mut!(data_len) as usize,
                },
                &ReadBuffer {
                    addr: raw_handles.as_mut_ptr() as usize,
                    len: raw_handles.len(),
                    actual_len_addr: addr_of_mut!(handle_count) as usize,
                },
            )?;
        }

        let mut handles = Vec::with_capacity(handle_count as usize);
        for i in 0..handle_count {
            handles.push(unsafe { OwnedHandle::from_raw(raw_handles[i as usize]) });
        }

        let data = raw_data[..data_len].to_vec();

        unsafe {
            dealloc(raw_handles.as_mut_ptr() as *mut u8, raw_handles_layout);
            dealloc(raw_data.as_mut_ptr(), data_layout);
        }
        Ok(MessagePacket { data, handles })
    }

    pub fn write(&self, packet: MessagePacket) -> Result<()> {
        let MessagePacket { data, handles } = packet;
        let mut raw_handles = Vec::with_capacity(handles.len());

        for handle in handles {
            raw_handles.push(unsafe { handle.as_raw() });
            core::mem::forget(handle);
        }

        unsafe {
            sys_write_channel(
                self.0.as_raw(),
                &WriteBuffer {
                    addr: data.as_ptr() as usize,
                    len: data.len(),
                },
                &WriteBuffer {
                    addr: raw_handles.as_ptr() as usize,
                    len: raw_handles.len(),
                },
            )?;
        }
        Ok(())
    }
}
