use bitflags::bitflags;
pub use vmar::*;
pub use vmo::*;

mod vmar;
mod vmo;

pub const PAGE_SIZE: usize = 4096;

bitflags! {
    /// Flags for mapping.
    #[repr(C)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    pub struct MMUFlags: u32 {
        const READ = 1 << 0;
        const WRITE = 1 << 1;
        const EXECUTE = 1 << 2;

        const DATA = Self::READ.bits() | Self::WRITE.bits();
        const CODE = Self::READ.bits() | Self::EXECUTE.bits();
        const RWX = Self::READ.bits() | Self::WRITE.bits() | Self::EXECUTE.bits();
    }
}
