use alloc::{string::String, vec, vec::Vec};
use raca_std::{
    fs::{File, OpenMode},
    kernel::insert_module,
    path::Path,
    println,
};

pub fn insmod(args: Vec<String>) {
    if args.len() != 1 {
        println!("Usage: insmod <module>");
        return;
    }

    let path = Path::new(args[0].clone());
    if let Ok(module) = File::open(path, OpenMode::Read) {
        let mut data = vec![0; module.size().unwrap()];
        module.read(&mut data).unwrap();
        insert_module(&data).unwrap();
    } else {
        println!("File not found!");
    }
}
