#![no_std]
#![no_main]

use raca_std::println;

#[unsafe(no_mangle)]
pub fn main() {
    let args = raca_std::env::args();
    if args.len() > 1 {
        println!("Usage: poweroff");
        return;
    }
    
    raca_std::kernel::poweroff();
    
    println!();
}
