use kernel_hal::mem::{MMUFlags, PageProperty};
use object::mem::Vmar;

fn main() {
    env_logger::init();
    kernel_hal::platform::init();

    let vmar = Vmar::new();
    vmar.activate();

    let (address, _) = vmar
        .map_with_alloc(4 * 1024, PageProperty::kernel_code())
        .unwrap();
    vmar.protect(address, 4 * 1024, MMUFlags::WRITE).unwrap();
}
