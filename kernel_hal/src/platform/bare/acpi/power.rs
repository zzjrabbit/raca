use acpi::address::MappedGas;

use crate::{arch::mem::flush_cache, mem::VirtAddr, platform::acpi::handler::AcpiHandler};

use super::ACPI;

pub fn reboot() -> ! {
    let reset_addr = ACPI.fadt.reset_register().unwrap();
    let reset = unsafe { MappedGas::map_gas(reset_addr, &AcpiHandler).unwrap() };
    log::debug!(
        "reset addr: {:#x?} value: {:#x?}",
        reset_addr,
        ACPI.fadt.reset_value
    );
    loop {
        reset.write(ACPI.fadt.reset_value as u64).unwrap();
        flush_cache(reset_addr.address as VirtAddr);
    }
}

pub fn shutdown() -> ! {
    crate::arch::idle_loop();
}
