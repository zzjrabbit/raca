#![no_std]
#![no_main]

#[no_mangle]
pub extern "C" fn main() -> usize {

    raca_std::debug::debug("Initial user program started");
    loop {
        raca_std::dummy();
    }
}

