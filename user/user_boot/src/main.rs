#![no_std]
#![no_main]
#![feature(iter_array_chunks)]

extern crate alloc;

use alloc::vec::Vec;
use protocol::{BOOT_FB_HANDLE_IDX, FB_HEIGHT_IDX, FB_WIDTH_IDX};
use ustd::{
    ipc::{Channel, MessagePacket},
    os::raca::OwnedHandle,
    vm::Vmo,
};

mod shell;
mod terminal;

#[unsafe(no_mangle)]
pub extern "Rust" fn main(channel: &Channel) -> i32 {
    let MessagePacket { data, handles } = channel.read().unwrap();
    let data = data
        .into_iter()
        .array_chunks()
        .map(|a| usize::from_le_bytes(a))
        .collect::<Vec<usize>>();

    let fb_width = data[FB_WIDTH_IDX];
    let fb_height = data[FB_HEIGHT_IDX];
    let fb_vmo = unsafe {
        Vmo::from_handle_count(
            OwnedHandle::from_raw(handles[BOOT_FB_HANDLE_IDX].as_raw()),
            fb_width * fb_height * 4,
        )
    };
    terminal::init(fb_vmo, fb_width, fb_height);
    termpln!("Hello World");

    core::mem::forget(handles);
    0
}
