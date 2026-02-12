use std::env;

fn main() {
    let target = env::var("TARGET").unwrap();
    let arch = target.split('-').next().unwrap();

    println!("cargo::rustc-link-search=target/x86_64-unknown-linux-none/debug");
    println!("cargo::rustc-link-lib=vdso_dylib");
}
