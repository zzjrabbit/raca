#![no_std]
#![no_main]

use raca_std::{print, println};

#[unsafe(no_mangle)]
pub fn main() {
    let args = raca_std::env::args();
    for arg in args.skip(1) {
        print!("{} ", arg);
    }
    println!();
}
