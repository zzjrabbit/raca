#![no_std]
#![no_main]

use ustd::ipc::Channel;

#[unsafe(no_mangle)]
pub extern "Rust" fn main(_channel: &Channel) -> i32 {
    ustd::println!("terminal in");
    0
}
