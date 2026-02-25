use anyhow::Result;

use crate::{cargo::CargoOpts, commands::target};

pub fn do_clippy() -> Result<()> {
    let run_clippy = |kcrate: CargoOpts| {
        let mut crate1 = kcrate.clone();
        crate1.target(target("loongarch64").into());
        crate1.done();

        let mut crate2 = kcrate.clone();
        crate2.feature("libos");
        crate2.done();
    };

    let mut object = CargoOpts::new("object".into());
    object.action("clippy");
    run_clippy(object);

    let mut kernel_hal = CargoOpts::new("kernel_hal".into());
    kernel_hal.action("clippy");
    run_clippy(kernel_hal);

    let mut kernel = CargoOpts::new("kernel".into());
    kernel.action("clippy");
    kernel.target(target("loongarch64").into());
    kernel.done();

    Ok(())
}
