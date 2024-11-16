use alloc::{string::String, vec::Vec, vec};
use raca_std::{fs::{File, OpenMode}, kernel::insert_module, path::Path, println};

pub fn insmod(args: Vec<String>) {
    if args.len() != 1 {
        println!("Usage: insmod <module>");
        return;
    }

    let path = Path::new(args[0].clone());
    if let Ok(module) = File::open(path, OpenMode::Read) {
        let mut data = vec![0; module.size() as usize];
        module.read(&mut data);
        insert_module(&data);
    } else {
        println!("File not found!");
    }
}