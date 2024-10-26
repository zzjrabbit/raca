#![no_std]
#![no_main]
#![feature(generic_const_exprs)]

use core::{ffi::CStr, panic::PanicInfo, slice};
use limine::{request::ModuleRequest, BaseRevision};
use raca_core::module::Module;

#[used]
#[link_section = ".requests"]
pub static BASE_REVISION: BaseRevision = BaseRevision::with_revision(1);

const fn create_string<const N: usize>(s: &[u8; N]) -> [u8; N + 1] {
    let mut res = [0; N + 1];
    let mut i = 0;
    while i < N {
        res[i] = s[i];
        i += 1;
    }
    res[N] = 0;
    res
}

static HELLO_MODULE: limine::modules::InternalModule = limine::modules::InternalModule::new()
    .with_path(unsafe {
        CStr::from_bytes_with_nul_unchecked(&create_string(b"/modules/hello.km"))
    });

#[used]
#[link_section = ".requests"]
static HELLO: ModuleRequest = ModuleRequest::new().with_internal_modules(&[&HELLO_MODULE]);

#[no_mangle]
pub extern "C" fn main() -> ! {
    raca_core::init();
    let module = HELLO.get_response().unwrap().modules()[0];
    let (ptr, size) = (module.addr(), module.size());
    let data = unsafe { slice::from_raw_parts_mut(ptr, size as usize) };
    let module = Module::load(data);
    module.init();
    drop(module);

    loop {
        x86_64::instructions::hlt();
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    log::error!("{}", info);
    loop {
        x86_64::instructions::hlt();
    }
}
