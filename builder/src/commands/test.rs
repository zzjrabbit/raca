use anyhow::Result;

use crate::cargo::CargoOpts;

pub fn do_test() -> Result<()> {
    let mut object = CargoOpts::new("object".into());
    object.action("test");
    object.feature("libos");
    object.done();

    let mut kernel_hal = CargoOpts::new("kernel_hal".into());
    kernel_hal.action("test");
    kernel_hal.feature("libos");
    kernel_hal.done();

    Ok(())
}
