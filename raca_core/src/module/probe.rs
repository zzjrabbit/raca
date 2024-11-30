use alloc::{sync::Arc, vec, vec::Vec};
use spin::Mutex;

use super::Module;
use crate::fs::Path;

static DRIVER_MODULES: Mutex<Vec<Arc<Module>>> = Mutex::new(Vec::new());

pub fn probe() {
    let modules_folder = crate::fs::operation::kernel_open(Path::new("/modules"))
        .expect("failed to open modules folder");
    let keyboard_driver_module_file = modules_folder
        .read()
        .get_child("keyboard.km")
        .expect("failed to open keyboard module");

    let mut keyboard_driver_module_data =
        vec![0; keyboard_driver_module_file.read().len() as usize];
    keyboard_driver_module_file
        .read()
        .read_at(0, &mut keyboard_driver_module_data);

    let keyboard_driver_module = Module::load(&keyboard_driver_module_data);
    keyboard_driver_module.init();

    DRIVER_MODULES.lock().push(keyboard_driver_module);
}
