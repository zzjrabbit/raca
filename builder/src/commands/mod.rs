use std::path::{Path, PathBuf};

use anyhow::Result;
pub use build::*;
pub use clippy::*;
pub use run::*;
pub use test::*;

use crate::cargo::CargoOpts;

mod build;
mod clippy;
mod run;
mod test;

fn target_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("target")
}

fn user_target(arch: &str) -> &str {
    match arch {
        "x86_64" => "x86_64-unknown-linux-none",
        "loongarch64" => "loongarch64-unknown-linux-musl",
        _ => unimplemented!(),
    }
}

fn build_vdso(target_dir: &Path, arch: &str, release: bool, libos: bool) -> Result<PathBuf> {
    let user_target = user_target(arch);

    let mut vdso_dylib = CargoOpts::new("vdso_dylib".into());
    vdso_dylib.build_std();
    vdso_dylib.target(user_target.into());

    if libos {
        vdso_dylib.feature("libos");
    }
    if release {
        vdso_dylib.release();
    }

    vdso_dylib.done();

    Ok(target_dir
        .join(user_target)
        .join(if release { "release" } else { "debug" })
        .join("libvdso_dylib.so"))
}

fn build_user_boot(
    target_dir: &Path,
    vdso_dylib_path: &Path,
    arch: &str,
    release: bool,
) -> Result<PathBuf> {
    let user_target = user_target(arch);

    let mut user_boot = CargoOpts::new("user_boot".into());
    user_boot.build_std();
    user_boot.target(user_target.into());

    if release {
        user_boot.release();
    }

    user_boot.env("VDSO_DYLIB_PATH", vdso_dylib_path.to_str().unwrap());

    user_boot.done();

    Ok(target_dir
        .join(user_target)
        .join(if release { "release" } else { "debug" })
        .join("user_boot"))
}
