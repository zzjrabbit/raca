use anyhow::Result;

use crate::{
    cargo::CargoOpts,
    commands::{build_vdso, target_dir},
};

pub fn do_test() -> Result<()> {
    let target_dir = target_dir();
    let arch = "x86_64";

    let vdso_dylib_path = build_vdso(&target_dir, arch, false, true)?;

    let mut object = CargoOpts::new("object".into());
    object.action("test");
    object.env("VDSO_DYLIB_PATH", vdso_dylib_path.to_str().unwrap());
    object.feature("libos");
    object.done();

    let mut kernel_hal = CargoOpts::new("kernel_hal".into());
    kernel_hal.action("test");
    kernel_hal.feature("libos");
    kernel_hal.done();

    Ok(())
}
