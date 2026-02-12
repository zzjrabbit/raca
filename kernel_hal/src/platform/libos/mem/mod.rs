pub use phys::*;
pub use vm::*;

mod info;
mod phys;
mod vm;

pub const KERNEL_ASPACE_BASE: usize = 0xffff_ff02_0000_0000;
pub const KERNEL_ASPACE_SIZE: usize = 0x0000_0080_0000_0000;
pub const USER_ASPACE_BASE: usize = 0x10000000;
pub const USER_ASPACE_SIZE: usize = 0x800000000 - USER_ASPACE_BASE;
