#![no_std]

use spin::Mutex;

extern crate alloc;

#[doc(hidden)]
pub mod io;
mod memory;

#[repr(C)]
#[allow(dead_code)]
pub struct InfoStruct {
    name: &'static str,
    license: &'static str,
}

impl InfoStruct {
    pub const fn with_name(name: &'static str) -> Self {
        Self {
            name,
            license: "MIT",
        }
    }

    pub const fn with_name_license(name: &'static str, license: &'static str) -> Self {
        Self { name, license }
    }

    pub const fn get_name(&self) -> &'static str {
        self.name
    }
}

static MODULE_HANDLE: Mutex<usize> = Mutex::new(0);

pub(crate) fn get_module_handle() -> usize {
    *MODULE_HANDLE.lock()
}

#[macro_export]
macro_rules! kernel_module {
    ($name: ident,$init: ident,$exit: ident) => {
        #[used]
        #[link_section = ".info"]
        #[no_mangle]
        static MODULE_INFO: module_std::InfoStruct =
            module_std::InfoStruct::with_name(stringify!($name));

        #[no_mangle]
        #[link_section = ".module"]
        pub extern "C" fn module_init() -> usize {
            $init();
            0
        }

        #[allow(dead_code)]
        #[no_mangle]
        #[link_section = ".module"]
        pub extern "C" fn module_exit() -> usize {
            $exit();
            0
        }
    };
    ($name: ident,$license: ident, $init: ident, $exit: ident) => {
        #[used]
        #[link_section = ".info"]
        #[no_mangle]
        static MODULE_INFO: module_std::InfoStruct =
            module_std::InfoStruct::with_name_license(stringify!($name), stringify!($license));

        #[no_mangle]
        #[link_section = ".module"]
        pub extern "C" fn module_init() -> usize {
            $init();
            0
        }

        #[allow(dead_code)]
        #[no_mangle]
        #[link_section = ".module"]
        pub extern "C" fn module_exit() -> usize {
            $exit();
            0
        }
    };
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    log::error!("{}", info);
    loop {}
}
