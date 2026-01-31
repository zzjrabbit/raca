#![cfg_attr(not(feature = "libos"), no_std)]

extern crate alloc;

pub mod io;
pub mod mem;

#[cfg(feature = "libos")]
#[path = "libos/mod.rs"]
pub mod platform;

#[cfg(not(feature = "libos"))]
#[path = "bare/mod.rs"]
pub mod platform;
