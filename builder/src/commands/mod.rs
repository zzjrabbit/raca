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

fn target(arch: &str) -> &str {
    match arch {
        "loongarch64" => "loongarch64-unknown-none-softfloat",
        "x86_64" => "x86_64-unknown-none",
        _ => panic!("Unsupported architecture: {}", arch),
    }
}

fn build_user_boot(target_dir: &Path, arch: &str, release: bool) -> Result<PathBuf> {
    let user_target = target(arch);

    let mut user_boot = CargoOpts::new("user_boot".into());
    user_boot.env("RUSTFLAGS", "-C relocation-model=pie");
    user_boot.target(user_target.into());

    if release {
        user_boot.release();
    }

    user_boot.done();

    Ok(target_dir
        .join(user_target)
        .join(if release { "release" } else { "debug" })
        .join("user_boot"))
}
