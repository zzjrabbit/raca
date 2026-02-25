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

static USER_PROGRAMS: &[&str] = &["terminal", "user_boot"];

fn build_user_programs(
    target_dir: &Path,
    arch: &str,
    release: bool,
) -> Result<Vec<(String, PathBuf)>> {
    let user_target = target(arch);

    let build_one = |name: String| -> Result<_> {
        let mut cargo = CargoOpts::new(name.clone());
        cargo.target(user_target.into());
        if release {
            cargo.release();
        }
        cargo.done();
        let path = target_dir
            .join(user_target)
            .join(if release { "release" } else { "debug" })
            .join(&name);
        Ok((name.to_string(), path))
    };

    let mut result = Vec::new();
    for program in USER_PROGRAMS {
        let program = program.to_string();
        result.push(build_one(program)?);
    }
    Ok(result)
}
