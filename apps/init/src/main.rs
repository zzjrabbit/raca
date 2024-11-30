#![no_std]
#![no_main]

use alloc::vec;
use raca_std::{fs::File, path::Path, task::Process};

extern crate alloc;

#[no_mangle]
pub extern "C" fn main() -> usize {
    let shell = File::open(Path::new("/bin/shell.rae"), raca_std::fs::OpenMode::Read).unwrap();

    let size = shell.size().unwrap();
    let mut data = vec![0; size as usize];

    shell.read(&mut data).unwrap();

    let process = Process::new(&data, "shell", 0, 1);
    process.run().unwrap();

    loop {
        raca_std::dummy();
    }
}
