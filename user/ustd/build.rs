use std::{env, path::PathBuf};

fn main() {
    let path = PathBuf::from(env::var("VDSO_DYLIB_PATH").unwrap());
    println!(
        "cargo::rustc-link-search={}",
        path.parent().unwrap().display()
    );
    println!("cargo::rustc-link-lib=vdso_dylib");
}
