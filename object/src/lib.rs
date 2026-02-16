#![no_std]
#![feature(get_mut_unchecked)]

extern crate alloc;

use errors::*;

pub mod ipc;
pub mod mem;
pub mod object;
pub mod task;

pub fn init() {
    task::init();
}
