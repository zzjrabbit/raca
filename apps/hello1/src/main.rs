#![no_std]
#![no_main]

#[no_mangle]
pub extern "C" fn main() -> usize {
    loop {
        raca_std::dummy();
    }
}

