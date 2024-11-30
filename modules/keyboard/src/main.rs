#![no_std]
#![no_main]
use module_std::{KernelModule, driver::end_of_interrupt, kernel_module, print, println};
use x86_64::structures::idt::InterruptStackFrame;

kernel_module!(Keyboard, keyboard, Apache);

pub struct Keyboard;

fn keyboard_handler(_frame: InterruptStackFrame) {
    let scan_code = unsafe { x86_64::instructions::port::PortReadOnly::new(0x60).read() };
    module_std::driver::append_keyboard_scan_code(scan_code);
    end_of_interrupt();
}

impl KernelModule for Keyboard {
    fn init() -> Option<Self> {
        let irq_handler = module_std::driver::IrqHandler::new(1, keyboard_handler);
        irq_handler.register();
        Some(Self)
    }
}

impl Drop for Keyboard {
    fn drop(&mut self) {
    }
}
