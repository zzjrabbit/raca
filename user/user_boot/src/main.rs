#![no_std]
#![no_main]

use ustd::ipc::Channel;

#[unsafe(no_mangle)]
pub extern "Rust" fn main(channel: &Channel) -> i32 {
    let msg = channel.read().unwrap();
    ustd::debug(core::str::from_utf8(&msg.data).unwrap()).unwrap();
    0
}
