use alloc::sync::Arc;
use bitflags::bitflags;
use spin::RwLock;

use errors::Error;

pub type PhysAddr = usize;
pub type VirtAddr = usize;

/// Page Size
/// Interfaces to support huge page.
#[repr(usize)]
#[derive(Default, Debug, Copy, Clone, Eq, PartialEq)]
pub enum PageSize {
    #[default]
    Size4K = 0x1000,
    Size2M = 0x20_0000,
    Size1G = 0x4000_0000,
}

impl TryFrom<usize> for PageSize {
    type Error = ();
    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value {
            0x1000 => Ok(Self::Size4K),
            0x20_0000 => Ok(Self::Size2M),
            0x4000_0000 => Ok(Self::Size1G),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Page {
    pub vaddr: VirtAddr,
    pub size: PageSize,
}

impl PageSize {
    pub const fn is_aligned(self, addr: usize) -> bool {
        self.page_offset(addr) == 0
    }

    pub const fn align_down(self, addr: usize) -> usize {
        addr & !(self as usize - 1)
    }

    pub const fn align_up(self, addr: usize) -> usize {
        self.align_down(addr + self as usize - 1)
    }

    pub const fn page_offset(self, addr: usize) -> usize {
        addr & (self as usize - 1)
    }

    pub const fn is_huge(self) -> bool {
        matches!(self, Self::Size1G | Self::Size2M)
    }
}

impl Page {
    pub fn new_aligned(vaddr: VirtAddr, size: PageSize) -> Self {
        Self { vaddr, size }
    }
}

bitflags! {
    /// Flags for mapping.
    #[repr(C)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    pub struct MMUFlags: u32 {
        const READ = 1 << 0;
        const WRITE = 1 << 1;
        const EXECUTE = 1 << 2;
        const HUGE_PAGE = 1 << 3;
    }
}

#[repr(C)]
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CachePolicy {
    #[default]
    CacheCoherent,
    StronglyOrderedUnCached,
    WeaklyOrderedUnCached,
}

#[repr(C)]
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Privilege {
    User,
    #[default]
    KernelOnly,
    UserOnly,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PageProperty {
    pub flags: MMUFlags,
    pub cache_policy: CachePolicy,
    pub privilege: Privilege,
}

impl PageProperty {
    pub fn new(flags: MMUFlags, cache_policy: CachePolicy, privilege: Privilege) -> Self {
        PageProperty {
            flags,
            cache_policy,
            privilege,
        }
    }

    pub fn kernel_code() -> Self {
        PageProperty {
            flags: MMUFlags::READ | MMUFlags::EXECUTE,
            cache_policy: CachePolicy::CacheCoherent,
            privilege: Privilege::KernelOnly,
        }
    }

    pub const fn kernel_data() -> Self {
        PageProperty {
            flags: MMUFlags::from_bits_truncate(MMUFlags::READ.bits() | MMUFlags::WRITE.bits()),
            cache_policy: CachePolicy::CacheCoherent,
            privilege: Privilege::KernelOnly,
        }
    }

    pub const fn user_code() -> Self {
        PageProperty {
            flags: MMUFlags::from_bits_truncate(MMUFlags::READ.bits() | MMUFlags::EXECUTE.bits()),
            cache_policy: CachePolicy::CacheCoherent,
            privilege: Privilege::User,
        }
    }

    pub fn user_data() -> Self {
        PageProperty {
            flags: MMUFlags::READ | MMUFlags::WRITE,
            cache_policy: CachePolicy::CacheCoherent,
            privilege: Privilege::User,
        }
    }
}

#[allow(dead_code)]
pub trait GeneralPageTable: Sync + Send {
    fn map(&mut self, page: Page, paddr: PhysAddr, property: PageProperty) -> Result<(), Error>;
    fn unmap(&mut self, vaddr: VirtAddr) -> Result<PageSize, Error>;
    fn update(&mut self, vaddr: VirtAddr, property: PageProperty) -> Result<PageSize, Error>;
    fn query(&mut self, vaddr: VirtAddr) -> Result<(PhysAddr, PageProperty, PageSize), Error>;

    fn deep_copy(&self) -> Arc<RwLock<dyn GeneralPageTable>>;
    fn activate(&self);

    /// Note that start_vaddr and size must be aligned by page size
    fn map_cont(
        &mut self,
        start_vaddr: VirtAddr,
        size: usize,
        start_paddr: PhysAddr,
        property: PageProperty,
    ) -> Result<(), Error> {
        let mut vaddr = start_vaddr;
        let mut paddr = start_paddr;
        let end_vaddr = vaddr + size;
        if property.flags.contains(MMUFlags::HUGE_PAGE) {
            while vaddr < end_vaddr {
                let remains = end_vaddr - vaddr;
                let page_size = if remains >= PageSize::Size1G as usize
                    && PageSize::Size1G.is_aligned(vaddr)
                    && PageSize::Size1G.is_aligned(paddr)
                {
                    PageSize::Size1G
                } else if remains >= PageSize::Size2M as usize
                    && PageSize::Size2M.is_aligned(vaddr)
                    && PageSize::Size2M.is_aligned(paddr)
                {
                    PageSize::Size2M
                } else {
                    PageSize::Size4K
                };
                let page = Page::new_aligned(vaddr, page_size);
                self.map(page, paddr, property)?;
                vaddr += page_size as usize;
                paddr += page_size as usize;
            }
        } else {
            while vaddr < end_vaddr {
                let page_size = PageSize::Size4K;
                let page = Page::new_aligned(vaddr, page_size);
                self.map(page, paddr, property)?;
                vaddr += page_size as usize;
                paddr += page_size as usize;
            }
        }
        Ok(())
    }

    /// Note that start_vaddr and size must be aligned by page size.
    fn unmap_cont(&mut self, start_vaddr: VirtAddr, size: usize) -> Result<(), Error> {
        let mut vaddr = start_vaddr;
        let end_vaddr = vaddr + size;
        while vaddr < end_vaddr {
            let page_size = match self.unmap(vaddr) {
                Ok(s) => {
                    assert!(s.is_aligned(vaddr));
                    s as usize
                }
                Err(e) => return Err(e),
            };
            vaddr += page_size;
            assert!(vaddr <= end_vaddr);
        }
        Ok(())
    }

    fn update_cont(
        &mut self,
        start_vaddr: VirtAddr,
        size: usize,
        property: PageProperty,
    ) -> Result<(), Error> {
        let mut vaddr = start_vaddr;
        let end_vaddr = vaddr + size;
        while vaddr < end_vaddr {
            let page_size = match self.update(vaddr, property) {
                Ok(s) => {
                    assert!(s.is_aligned(vaddr));
                    s as usize
                }
                Err(e) => return Err(e),
            };
            vaddr += page_size;
            assert!(vaddr <= end_vaddr);
        }
        Ok(())
    }
}
