#![no_std]
#![no_main]

use core::time::Duration;

#[no_mangle]
pub extern "C" fn main() -> usize {
    raca_std::dummy();
    raca_std::thread::sleep(Duration::from_secs(1));
    0
}
