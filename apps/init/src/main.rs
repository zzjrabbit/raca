#![no_std]
#![no_main]

#[no_mangle]
pub extern "C" fn main() -> usize {

    raca_std::debug::debug("Initial user program started");
    raca_std::fs::open("/bin/init.rae", raca_std::fs::OpenMode::Read);
    loop {
        raca_std::dummy();
    }
}

