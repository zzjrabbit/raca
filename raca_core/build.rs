use std::path::PathBuf;

fn main() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    // Tell cargo to pass the linker script to the linker..
    println!(
        "cargo:rustc-link-arg=-T{}/linker-{arch}.ld",
        manifest_dir.display()
    );
    // ..and to re-run if it changes.
    println!(
        "cargo:rerun-if-changed={}/linker-{arch}.ld",
        manifest_dir.display()
    );

    println!(
        "cargo:rerun-if-changed={}/src/syscall/syscall_numbers",
        manifest_dir.display()
    );

}
