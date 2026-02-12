use std::env;

fn main() {
    let target = env::var("TARGET").unwrap();
    let arch = target.split('-').next().unwrap();
    
    let target = match arch {
        "x86_64" => "x86_64-unknown-linux-none",
        "loongarch64" => "loongarch64-unknown-linux-musl",
        _ => panic!("Unsupported arch {}!", arch),
    };

    println!("cargo::rustc-link-search=target/{}/debug", target);
    println!("cargo::rustc-link-lib=vdso_dylib");
}
