use core::panic::PanicInfo;

#[panic_handler]
fn panic_handler(info: &PanicInfo) -> ! {
    log::error!("Panic: {}", info);
    crate::arch::idle_loop()
}
