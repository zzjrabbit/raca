#![no_std]
#![no_main]

#[unsafe(no_mangle)]
pub extern "C" fn main() -> i32 {
    ustd::dummy();
    0
}
