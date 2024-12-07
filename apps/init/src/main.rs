#![no_std]
#![no_main]

use alloc::{collections::btree_map::BTreeMap, string::ToString};
use raca_std::{path::Path, process::Command};

extern crate alloc;

#[no_mangle]
pub extern "C" fn main() -> usize {
    raca_std::env::set_env({
        let mut env = BTreeMap::new();
        env.insert("PATH".to_string(), "/bin".to_string());
        env
    });

    let shell = Command::new(Path::new("/bin/shell.rae"));
    shell.spawn().unwrap();

    loop {
        raca_std::dummy();
    }
}
