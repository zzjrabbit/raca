use x86_64::structures::idt::InterruptStackFrame;

pub type IrqHandleFunction = fn(frame: InterruptStackFrame);

pub struct IrqHandler {
    pub irq: u8,
    pub handle_function: IrqHandleFunction,
}

extern "C" {
    fn add_interrupt_handler(handler: u64) -> u8;
    fn enable_irq_to(irq: u8, vector: u8);
    fn add_keyboard_scan_code(scan_code: u8);
}

pub fn append_keyboard_scan_code(scan_code: u8) {
    unsafe {
        add_keyboard_scan_code(scan_code);
    }
}

pub fn end_of_interrupt() {
    extern "C" {
        fn end_of_interrupt();
    }
    unsafe {
        end_of_interrupt();
    }
}

impl IrqHandler {
    pub fn new(irq: u8, handle_function: IrqHandleFunction) -> Self {
        Self {
            irq,
            handle_function,
        }
    }

    pub fn register(&self) {
        unsafe {
            let vector = add_interrupt_handler(self.handle_function as u64);
            enable_irq_to(self.irq, vector);
        }
    }
}
