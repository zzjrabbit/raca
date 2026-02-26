#![no_std]
#![no_main]
#![feature(iter_array_chunks)]
#![feature(allocator_api)]

extern crate alloc;

use alloc::vec::Vec;
use pci_types::device_type::DeviceType;
use protocol::{
    BOOT_FB_HANDLE_IDX, BOOT_PCIE_HANDLE_IDX, FB_HEIGHT_IDX, FB_WIDTH_IDX, PCIE_INFO_LEN_IDX,
};
use spin::{Lazy, Once};
use ustd::{
    ipc::{Channel, MessagePacket},
    os::raca::OwnedHandle,
    vm::Vmo,
};

use crate::pcie::PCI_DEVICES;

mod kbd;
mod pcie;
mod shell;
mod terminal;

static PCIE_INFO_VMO: Once<Vmo> = Once::new();

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
        Vmo::from_handle_len(
            OwnedHandle::from_raw(handles[BOOT_FB_HANDLE_IDX].as_raw()),
            fb_width * fb_height * 4,
        )
    };
    terminal::init(fb_vmo, fb_width, fb_height);

    termpln!("Hello World");

    let pcie_info_len = data[PCIE_INFO_LEN_IDX];
    let pcie_info_vmo = unsafe {
        Vmo::from_handle_len(
            OwnedHandle::from_raw(handles[BOOT_PCIE_HANDLE_IDX].as_raw()),
            pcie_info_len,
        )
    };
    PCIE_INFO_VMO.call_once(|| pcie_info_vmo);
    Lazy::force(&pcie::PCI_DEVICES);

    let devices = PCI_DEVICES.lock();
    let device = devices
        .iter()
        .find(|d| d.device_type == DeviceType::KeyboardController)
        .unwrap();
    termpln!("device bars: {:x?}", device.bars);
    let bar = device.bars[4].unwrap();
    let (address, size) = bar.unwrap_mem();
    let vmo = Vmo::acquire(address, size).unwrap();
    kbd::init(vmo);

    core::mem::forget(handles);
    0
}
