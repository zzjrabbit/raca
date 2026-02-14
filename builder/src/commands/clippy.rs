use anyhow::Result;

use crate::{
    cargo::CargoOpts,
    commands::{build_user_boot, build_vdso, target_dir},
};

pub fn do_clippy() -> Result<()> {
    let target_dir = target_dir();
    let arch = "x86_64";

    let vdso_dylib_path = build_vdso(&target_dir, arch, false, true)?;
    let user_boot_path = build_user_boot(&target_dir, &vdso_dylib_path, arch, false)?;

    let run_clippy = |kcrate: CargoOpts| {
        let mut crate1 = kcrate.clone();
        crate1.target("loongarch64-unknown-none".into());
        crate1.done();

        let mut crate2 = kcrate.clone();
        crate2.feature("libos");
        crate2.done();
    };

    let mut object = CargoOpts::new("object".into());
    object.action("clippy");
    object.env("VDSO_DYLIB_PATH", vdso_dylib_path.to_str().unwrap());
    run_clippy(object);

    let mut kernel_hal = CargoOpts::new("kernel_hal".into());
    kernel_hal.action("clippy");
    run_clippy(kernel_hal);

    let mut kernel = CargoOpts::new("kernel".into());
    kernel.action("clippy");
    kernel.target("loongarch64-unknown-none".into());
    kernel.env("VDSO_DYLIB_PATH", vdso_dylib_path.to_str().unwrap());
    kernel.env("USER_BOOT_PATH", user_boot_path.to_str().unwrap());
    kernel.done();

    Ok(())
}
