#![no_std]
#![no_main]

#[no_mangle]
pub extern "C" fn main() -> usize {
    raca_std::dummy();
    1
}
