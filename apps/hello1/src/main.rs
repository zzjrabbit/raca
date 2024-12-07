#![no_std]
#![no_main]

use core::time::Duration;

use raca_std::println;

fn thread() {
    raca_std::thread::sleep(Duration::from_secs(1));
    println!("done");
    loop {}
}

#[no_mangle]
pub extern "C" fn main() -> usize {
    //raca_std::dummy();

    raca_std::thread::spawn(thread).unwrap();
    println!("spawn");

    raca_std::thread::sleep(Duration::from_secs(2));
    println!("done main");
    //loop {}
    0
}
