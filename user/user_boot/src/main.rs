#![no_std]
#![no_main]

#[unsafe(no_mangle)]
pub extern "C" fn main() -> i32 {
    //ustd::debug("Hello World!");
    ustd::dummy();
    let msg = "Hello World!";
    unsafe {
        core::arch::asm!("syscall 0", in("$r4") msg.as_ptr() as usize, in("$r5") msg.len());
    }
    0
}
