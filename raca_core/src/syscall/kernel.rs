use core::slice::from_raw_parts;

use alloc::{sync::Arc, vec::Vec};
use spin::RwLock;

use crate::module::Module;

use crate::error::*;

#[used]
static USER_MODULES: RwLock<Vec<Arc<Module>>> = RwLock::new(Vec::new());

pub fn insert_module(module_addr: usize, module_len: usize) -> Result<usize> {
    let data = unsafe { from_raw_parts(module_addr as *const u8, module_len) };

    let module = Module::load(data);

    let ret = module.init();

    USER_MODULES.write().push(module);

    Ok(ret)
}

pub fn reboot() -> Result<usize> {
    crate::arch::acpi::reboot();
    Ok(0)
}

pub fn poweroff() -> Result<usize> {
    crate::arch::acpi::poweroff();
    Ok(0)
}
