#![no_std]
#![no_main]

use alloc::string::ToString;
use raca_std::{path::Path, process::Command};

extern crate alloc;

#[no_mangle]
pub extern "C" fn main() -> usize {
    raca_std::env::set_var("PATH".to_string(), "/bin".to_string());

    let shell = Command::new(Path::new("/bin/rash"));
    shell.spawn().unwrap();

    loop {
        raca_std::dummy();
    }
}
