#![no_std]
#![no_main]

use raca_std::println;

#[unsafe(no_mangle)]
pub fn main() -> usize {
    let args = raca_std::env::args();
    if args.len() > 1 {
        println!("Usage: reboot");
        return 0;
    }
    
    raca_std::kernel::reboot();
    
    println!();
    0
}
