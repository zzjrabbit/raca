#![allow(improper_ctypes_definitions)]

use core::alloc::Layout;

use alloc::{collections::btree_map::BTreeMap, string::String};
use spin::Lazy;

pub static KERNEL_SYMBOL_TABLE: Lazy<BTreeMap<String, u64>> = Lazy::new(|| {
    let mut symbol_table = BTreeMap::new();
    symbol_table.insert("print".into(), print as u64);
    symbol_table.insert("alloc".into(), alloc as u64);
    symbol_table.insert("dealloc".into(), dealloc as u64);
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
