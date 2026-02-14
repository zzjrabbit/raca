use std::path::PathBuf;

use anyhow::Result;

use crate::{
    BuildArgs,
    cargo::CargoOpts,
    commands::{build_user_boot, build_vdso, target_dir},
    image,
};

pub fn do_build(args: BuildArgs) -> Result<(PathBuf, PathBuf)> {
    let target_dir = target_dir();

    let BuildArgs { release, arch } = args;

    let vdso_dylib_path = build_vdso(&target_dir, &arch, release, false)?;
    let user_boot_path = build_user_boot(&target_dir, &vdso_dylib_path, &arch, release)?;

    let kernel_target = format!("{}-unknown-none", arch);
    let mut kernel = CargoOpts::new("kernel".into());
    kernel.target(kernel_target.clone());
    if release {
        kernel.release();
    }
    kernel.env("VDSO_DYLIB_PATH", vdso_dylib_path.to_str().unwrap());
    kernel.env("USER_BOOT_PATH", user_boot_path.to_str().unwrap());
    kernel.done();
    let kernel_path = target_dir
        .join(kernel_target)
        .join(if release { "release" } else { "debug" })
        .join("kernel");

    image::build(&kernel_path).map(|p| (kernel_path, p))
}
