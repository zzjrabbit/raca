#![cfg_attr(not(feature = "libos"), no_std)]

extern crate alloc;

pub use imp::*;
pub use page_table::*;

mod page_table;

#[cfg(feature = "libos")]
#[path = "libos/mod.rs"]
mod imp;

#[cfg(not(feature = "libos"))]
#[path = "bare/mod.rs"]
mod imp;
