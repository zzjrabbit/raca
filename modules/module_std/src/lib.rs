#![no_std]

extern crate alloc;

pub mod driver;
#[doc(hidden)]
pub mod io;
mod memory;

#[repr(C)]
#[allow(dead_code)]
pub struct ModuleInfo {
    name: &'static str,
    license: &'static str,
}

impl ModuleInfo {
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

pub trait KernelModule: Sized + Sync {
    fn init() -> Option<Self>;
}

#[macro_export]
macro_rules! kernel_module {
    ($module: ty, $name: ident ,$license: ident) => {
        #[used]
        #[link_section = ".info"]
        #[no_mangle]
        static MODULE_INFO: $crate::ModuleInfo =
            $crate::ModuleInfo::with_name_license(stringify!($name), stringify!($license));

        static mut __MODULE: Option<$module> = None;

        #[no_mangle]
        #[link_section = ".module"]
        pub extern "C" fn module_init() -> usize {
            if let Some(module) = <$module as $crate::KernelModule>::init() {
                unsafe {
                    __MODULE = Some(module);
                }
                0
            } else {
                0xff
            }
        }

        #[allow(dead_code)]
        #[no_mangle]
        #[link_section = ".module"]
        pub extern "C" fn module_exit() -> usize {
            unsafe {
                __MODULE = None;
            }
            0
        }
    };
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    log::error!("{}", info);
    loop {}
}
