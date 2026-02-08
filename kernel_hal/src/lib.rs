#![cfg_attr(not(feature = "libos"), no_std)]

extern crate alloc;

pub mod io;
pub mod mem;
pub mod task;

#[cfg(target_arch = "x86_64")]
#[path = "arch/x86_64/mod.rs"]
pub mod arch;

#[cfg(target_arch = "loongarch64")]
#[path = "arch/loongarch64/mod.rs"]
pub mod arch;

#[cfg(feature = "libos")]
#[path = "platform/libos/mod.rs"]
pub mod platform;

#[cfg(not(feature = "libos"))]
#[path = "platform/bare/mod.rs"]
pub mod platform;
