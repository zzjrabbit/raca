#![feature(exit_status_error)]

use std::{env, process::Command};

fn main() {
    /*let mut cargo = Command::new("cargo");
    cargo.arg("build").arg("--release");

    cargo.arg("-p").arg("vdso_dylib");
    cargo.arg("--target").arg(target);

    cargo.arg("-Zbuild-std");

    cargo.arg("--no-default-features");
    if env::var("CARGO_FEATURE_LIBOS").is_ok() {
        cargo.arg("--features").arg("libos");
    }

    cargo
        .spawn()
        .unwrap()
        .wait()
        .unwrap()
        .exit_ok()
        .expect("Failed to build vdso!");*/

    let target_triple = env::var("TARGET").unwrap();
    let target_arch = target_triple.split("-").next().unwrap();
    let target = match target_arch {
        "x86_64" => "x86_64-unknown-linux-none",
        "loongarch64" => "loongarch64-unknown-linux-musl",
        _ => panic!("Unsupported target architecture: {}", target_arch),
    };

    let vdso_path = format!("target/{}/debug/libvdso_dylib.so", target);
    println!("cargo::rustc-env=VDSO_PATH={}", vdso_path);
}
