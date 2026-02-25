use std::path::PathBuf;

use anyhow::Result;

use crate::{
    BuildArgs,
    cargo::CargoOpts,
    commands::{build_user_programs, target, target_dir},
    image,
};

pub fn do_build(args: BuildArgs) -> Result<(PathBuf, PathBuf)> {
    let target_dir = target_dir();

    let BuildArgs { release, arch } = args;

    let user_programs = build_user_programs(&target_dir, &arch, release)?;

    let kernel_target = target(&arch).to_string();
    let mut kernel = CargoOpts::new("kernel".into());
    kernel.target(kernel_target.clone());
    if release {
        kernel.release();
    }
    kernel.env("RUSTFLAGS", "-C relocation-model=static");
    kernel.done();
    let kernel_path = target_dir
        .join(kernel_target)
        .join(if release { "release" } else { "debug" })
        .join("kernel");

    image::build(&kernel_path, user_programs).map(|p| (kernel_path, p))
}
