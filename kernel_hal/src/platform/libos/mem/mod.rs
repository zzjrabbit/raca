pub use phys::*;
pub use vm::*;

mod info;
mod phys;
mod vm;

pub const KERNEL_ASPACE_BASE: usize = 0xffff_ff02_0000_0000;
pub const KERNEL_ASPACE_SIZE: usize = 0x0000_0080_0000_0000;
pub const USER_ASPACE_BASE: usize = 0x1000000000;
pub const USER_ASPACE_SIZE: usize = (1usize << 47) - 4096 - USER_ASPACE_BASE;
