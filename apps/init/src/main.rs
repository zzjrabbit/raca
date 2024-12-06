#![no_std]
#![no_main]

use raca_std::{path::Path, process::Command};

extern crate alloc;

#[no_mangle]
pub extern "C" fn main() -> usize {
    let shell = Command::new(Path::new("/bin/shell.rae"));
    shell.spawn().unwrap();

    loop {
        raca_std::dummy();
    }
}
