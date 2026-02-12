use std::{env, fs::File, path::PathBuf};

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    std::io::copy(
        &mut File::open("../syscalls.rs").unwrap(),
        &mut File::create(out_dir.join("syscalls.rs")).unwrap(),
    )
    .unwrap();
    println!("cargo:rerun-if-changed=../syscalls.rs");
}
