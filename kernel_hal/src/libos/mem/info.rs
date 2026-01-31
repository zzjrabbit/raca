use errors::{Errno, Result};

use crate::mem::{PageProperty, PageSize, PhysAddr, VirtAddr};

pub struct MemoryRegion {
    start: VirtAddr,
    size: usize,
    property: PageProperty,
    paddr: PhysAddr,
}

impl MemoryRegion {
    pub fn new(start: VirtAddr, size: usize, paddr: PhysAddr, property: PageProperty) -> Self {
        MemoryRegion {
            start,
            size,
            property,
            paddr,
        }
    }
}

impl MemoryRegion {
    pub fn vaddr(&self) -> VirtAddr {
        self.start
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn paddr(&self) -> PhysAddr {
        self.paddr
    }

    pub fn property(&self) -> PageProperty {
        self.property
    }

    pub fn set_property(&mut self, property: PageProperty) {
        self.property = property;
    }

    pub fn contains(&self, addr: VirtAddr) -> bool {
        self.start <= addr && addr < self.start + self.size
    }

    pub fn split_at(self, addr: VirtAddr) -> Result<(Self, Self)> {
        if !self.contains(addr) || !addr.is_multiple_of(PageSize::Size4K as usize) {
            return Err(Errno::InvArg.no_message());
        }

        let end = self.start + self.size;
        let offset = addr - self.start;

        let left = Self::new(self.start, offset, self.paddr, self.property);
        let right = Self::new(addr, end - addr, self.paddr + offset, self.property);

        Ok((left, right))
    }

    pub fn split_range(
        self,
        left: VirtAddr,
        right: VirtAddr,
    ) -> Result<(Option<Self>, Self, Option<Self>)> {
        if !left.is_multiple_of(PageSize::Size4K as usize)
            || !right.is_multiple_of(PageSize::Size4K as usize)
        {
            return Err(Errno::InvArg.no_message());
        }

        let start = self.start;
        let end = start + self.size();

        if left <= start && right >= end {
            Ok((None, self, None))
        } else if start < left {
            let (left, within) = self.split_at(left)?;
            if right < end {
                let (within, right) = within.split_at(right)?;
                Ok((Some(left), within, Some(right)))
            } else {
                Ok((Some(left), within, None))
            }
        } else if right < end {
            let (within, right) = self.split_at(right)?;
            Ok((None, within, Some(right)))
        } else {
            Err(Errno::InvArg.with_message("Memory region does not contain range."))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_range() {
        let region = MemoryRegion::new(0x1000, 0x1000, 0x2000, PageProperty::kernel_data());
        let (left, within, right) = region.split_range(0x1000, 0x2000).unwrap();
        assert!(left.is_none());
        assert_eq!(within.start, 0x1000);
        assert_eq!(within.size(), 0x1000);
        assert_eq!(within.paddr(), 0x2000);
        assert!(right.is_none());
    }

    #[test]
    fn test_split_at() {
        const START: usize = 0x10000;
        const SIZE: usize = 0x10000;
        const PADDR: usize = 0x20000;
        const PROPERTY: PageProperty = PageProperty::kernel_data();

        let region = MemoryRegion::new(START, SIZE, PADDR, PROPERTY);
        let (left, within) = region.split_at(START + SIZE / 2).unwrap();
        assert_eq!(left.start, START);
        assert_eq!(left.size(), SIZE / 2);
        assert_eq!(left.paddr(), PADDR);
        assert_eq!(within.start, START + SIZE / 2);
        assert_eq!(within.size(), SIZE / 2);
        assert_eq!(within.paddr(), PADDR + SIZE / 2);
    }
}
