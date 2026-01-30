use bitflags::bitflags;

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct Rights: u32 {
        const READ      = 1 << 0;
        const WRITE     = 1 << 1;
        const EXECUTE   = 1 << 2;
        const MAP       = 1 << 3;
        const DUPLICATE = 1 << 4;
        const TRANSFER  = 1 << 5;
        const WAIT      = 1 << 6;
        const SIGNAL    = 1 << 7;
        const MANAGE    = 1 << 8;

        const BASIC = Self::READ.bits() | Self::WRITE.bits() | Self::WAIT.bits();
        const ALL = u32::MAX;
    }
}
