#![no_std]
#![no_main]

use alloc::vec;
use raca_std::{fs::OpenMode, path::Path, println};

extern crate alloc;

#[unsafe(no_mangle)]
pub fn main() {
    let mut args = raca_std::env::args();
    if args.len() != 2 {
        println!("Usage: insmod <module>");
        return;
    }
    
    let _exe = args.next().unwrap();
    let module = args.next().unwrap();
    
    let Ok(file) = raca_std::fs::File::open(Path::new(module.clone()), OpenMode::Read) else {
        println!("Cannot open module {}", module);
        return;
    };
    
    let mut data = vec![0;file.size().unwrap()];
    file.read(&mut data).unwrap();
    
    raca_std::kernel::insert_module(&data).unwrap();
    
    println!();
}
