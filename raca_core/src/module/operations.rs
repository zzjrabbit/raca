#![allow(improper_ctypes_definitions)]

use core::alloc::Layout;

use alloc::{collections::btree_map::BTreeMap, string::String};
use spin::Lazy;

use crate::device::terminal::TERMINAL;

pub static KERNEL_SYMBOL_TABLE: Lazy<BTreeMap<String, u64>> = Lazy::new(|| {
    let mut symbol_table = BTreeMap::new();
    symbol_table.insert("print".into(), print as u64);
    symbol_table.insert("alloc".into(), alloc as u64);
    symbol_table.insert("dealloc".into(), dealloc as u64);
    symbol_table.insert("add_interrupt_handler".into(), add_interrupt_handler as u64);
    symbol_table.insert("enable_irq_to".into(), enable_irq_to as u64);
    symbol_table.insert("add_keyboard_scan_code".into(), add_keyboard_scan_code as u64);
    symbol_table.insert("end_of_interrupt".into(), end_of_interrupt as u64);
    symbol_table
});

pub extern "C" fn print(msg: &str) -> usize {
    crate::print!("{}", msg);
    0
}

pub extern "C" fn alloc(layout: Layout) -> *mut u8 {
    unsafe { alloc::alloc::alloc(layout) }
}

pub extern "C" fn dealloc(ptr: *mut u8, layout: Layout) {
    unsafe { alloc::alloc::dealloc(ptr, layout) }
}

pub extern "C" fn add_interrupt_handler(handler: u64) -> u8 {
    crate::arch::interrupts::add_interrupt_handler(unsafe { core::mem::transmute(handler) })
}

pub extern "C" fn enable_irq_to(irq: u8, vector: u8) {
    //log::info!("enabling irq {} to {}", irq, vector);
    unsafe {
        crate::arch::apic::ioapic_add_entry(irq, vector);
    }
}

pub extern "C" fn add_keyboard_scan_code(scan_code: u8) {
    let string_option = TERMINAL.lock().handle_keyboard(scan_code).clone();
    if let Some(string) = string_option {
        let mut keyboard_input = crate::fs::dev::KEYBOARD_INPUT.lock();
        keyboard_input.push_str(&string);
    }
}

pub extern "C" fn end_of_interrupt() {
    crate::arch::apic::end_of_interrupt();
}
