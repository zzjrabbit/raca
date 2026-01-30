#![no_std]
#![feature(get_mut_unchecked)]

extern crate alloc;

pub use error::*;

mod error;
pub mod ipc;
pub mod object;
pub mod task;
pub mod mem;
