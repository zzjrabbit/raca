use alloc::{sync::Arc, vec::Vec};
use errors::{Errno, Result};
use loongarch64::{
    PhysAddr as Paddr, PrivilegeLevel, VirtAddr as Vaddr,
    registers::{PgdHigh, PgdLow},
    structures::paging::{
        CachePolicy, FrameAllocator, FrameDeallocator, Mapper, OffsetPageTable, Page, PageProperty,
        PageSize, PageTable, PageTableFlags, PhysFrame, Size1GiB, Size2MiB, Size4KiB, Translate,
        TranslateResult,
    },
};
use spin::RwLock;

use crate::{
    mem::{
        BitmapFrameAllocator, GeneralPageTable, MMUFlags, PhysAddr, Privilege, VirtAddr,
        phys_to_virt, virt_to_phys,
    },
    platform::mem::FRAME_ALLOCATOR,
};

pub const KERNEL_ASPACE_BASE: usize = 0xffff_ff02_0000_0000;
pub const KERNEL_ASPACE_SIZE: usize = 0x0000_0080_0000_0000;
pub const USER_ASPACE_BASE: usize = 0x10_0000;
pub const USER_ASPACE_SIZE: usize = (1usize << 47) - 4096 - USER_ASPACE_BASE;

pub fn current_page_table() -> OffsetPageTable<'static> {
    let lower_half = PgdLow.read();
    let lower_half = phys_to_virt(lower_half as PhysAddr) as *mut PageTable;

    let higher_half = PgdHigh.read();
    let higher_half = phys_to_virt(higher_half as PhysAddr) as *mut PageTable;

    let physical_memory_offset = phys_to_virt(0) as u64;
    unsafe { OffsetPageTable::new(&mut *lower_half, &mut *higher_half, physical_memory_offset) }
}

fn kernel_property_converter(property: crate::mem::PageProperty) -> PageProperty {
    let mut result = PageProperty::new();

    let flags = property.flags;
    if !flags.contains(MMUFlags::READ) {
        result.add_flags(PageTableFlags::NO_READ);
    }
    if flags.contains(MMUFlags::WRITE) {
        result.add_flags(PageTableFlags::WRITABLE);
        result.add_flags(PageTableFlags::DIRTY);
    }
    if !flags.contains(MMUFlags::EXECUTE) {
        result.add_flags(PageTableFlags::NO_EXECUTE);
    }
    if flags.contains(MMUFlags::HUGE_PAGE) {
        result.add_flags(PageTableFlags::HUGE_PAGE);
        result.add_flags(PageTableFlags::GLOBAL_FOR_HUGE_PAGE);
    } else {
        result.add_flags(PageTableFlags::GLOBAL);
    }

    match property.privilege {
        Privilege::User => {
            result.set_privilege(PrivilegeLevel::Privilege3);
            result.set_privilege_restriction(false);
        }
        Privilege::KernelOnly => {
            result.set_privilege(PrivilegeLevel::Privilege0);
            result.set_privilege_restriction(true);
        }
        Privilege::UserOnly => {
            result.set_privilege(PrivilegeLevel::Privilege3);
            result.set_privilege_restriction(true);
        }
    }

    match property.cache_policy {
        crate::mem::CachePolicy::CacheCoherent => {
            result.set_cache_policy(CachePolicy::CoherentCached);
        }
        crate::mem::CachePolicy::WeaklyOrderedUnCached => {
            result.set_cache_policy(CachePolicy::WeaklyOrderedUnCached);
        }
        crate::mem::CachePolicy::StronglyOrderedUnCached => {
            result.set_cache_policy(CachePolicy::StronglyOrderedUnCached);
        }
    }

    result
}

fn loongarch64_property_converter(
    property: PageProperty,
    huge_page: bool,
) -> crate::mem::PageProperty {
    let mut flags = MMUFlags::empty();

    let source_flags = property.flags();
    if !source_flags.contains(PageTableFlags::NO_READ) {
        flags |= MMUFlags::READ;
    }
    if source_flags.contains(PageTableFlags::DIRTY) {
        flags |= MMUFlags::WRITE;
    }
    if !source_flags.contains(PageTableFlags::NO_EXECUTE) {
        flags |= MMUFlags::EXECUTE;
    }
    if huge_page {
        flags |= MMUFlags::HUGE_PAGE;
    }

    let cache_policy = match property.cache_policy() {
        CachePolicy::CoherentCached => crate::mem::CachePolicy::CacheCoherent,
        CachePolicy::WeaklyOrderedUnCached => crate::mem::CachePolicy::WeaklyOrderedUnCached,
        CachePolicy::StronglyOrderedUnCached => crate::mem::CachePolicy::StronglyOrderedUnCached,
        CachePolicy::Reserved => unreachable!(),
    };
    let privilege = {
        let privilege_level = property.privilege();
        let privilege_restriction = property.privilege_restriction();
        match privilege_level {
            PrivilegeLevel::Privilege0 => Privilege::KernelOnly,
            PrivilegeLevel::Privilege3 => {
                if privilege_restriction {
                    Privilege::UserOnly
                } else {
                    Privilege::User
                }
            }
            _ => unreachable!(),
        }
    };

    crate::mem::PageProperty::new(flags, cache_policy, privilege)
}

impl GeneralPageTable for OffsetPageTable<'_> {
    fn map(
        &mut self,
        page: crate::mem::Page,
        paddr: PhysAddr,
        property: crate::mem::PageProperty,
    ) -> Result<()> {
        let vaddr = Vaddr::new(page.vaddr as u64);
        let paddr = Paddr::new(paddr as u64);

        macro_rules! map_with_size {
            ($size: ident) => {{
                let page = Page::<$size>::containing_address(vaddr);
                let frame = PhysFrame::<$size>::containing_address(paddr);
                unsafe {
                    self.map_to(
                        page,
                        frame,
                        kernel_property_converter(property),
                        &mut *FRAME_ALLOCATOR.lock(),
                    )
                    .map_err(|_| errors::Errno::MapFailed.no_message())?
                    .flush();
                }
                Ok(())
            }};
        }

        match page.size {
            crate::mem::PageSize::Size4K => map_with_size!(Size4KiB),
            crate::mem::PageSize::Size2M => map_with_size!(Size2MiB),
            crate::mem::PageSize::Size1G => map_with_size!(Size1GiB),
        }
    }

    fn unmap(&mut self, vaddr: VirtAddr) -> Result<crate::mem::PageSize> {
        use loongarch64::structures::paging::Mapper;
        match self.translate(Vaddr::new(vaddr as u64)) {
            TranslateResult::Mapped { frame, .. } => {
                let size = crate::mem::PageSize::try_from(frame.size() as usize).unwrap();

                let vaddr = Vaddr::new(vaddr as u64);

                match frame.size() {
                    Size4KiB::SIZE => {
                        Mapper::unmap(self, Page::<Size4KiB>::containing_address(vaddr))
                            .unwrap()
                            .1
                            .flush()
                    }
                    Size2MiB::SIZE => {
                        Mapper::unmap(self, Page::<Size2MiB>::containing_address(vaddr))
                            .unwrap()
                            .1
                            .flush()
                    }
                    Size1GiB::SIZE => {
                        Mapper::unmap(self, Page::<Size1GiB>::containing_address(vaddr))
                            .unwrap()
                            .1
                            .flush()
                    }
                    _ => unreachable!(),
                }

                Ok(size)
            }
            TranslateResult::NotMapped => {
                Err(Errno::InvArg.with_message("Translating unmapped page."))
            }
            TranslateResult::InvalidFrameAddress(_) => {
                Err(Errno::InvArg.with_message("Invalid frame address."))
            }
        }
    }

    fn query(
        &mut self,
        vaddr: VirtAddr,
    ) -> Result<(PhysAddr, crate::mem::PageProperty, crate::mem::PageSize)> {
        match self.translate(Vaddr::new(vaddr as u64)) {
            TranslateResult::Mapped {
                frame,
                offset,
                property,
            } => {
                let address = frame.start_address().as_u64() as PhysAddr;
                let property =
                    loongarch64_property_converter(property, frame.size() != Size4KiB::SIZE);

                let size = match frame.size() {
                    Size4KiB::SIZE => crate::mem::PageSize::Size4K,
                    Size2MiB::SIZE => crate::mem::PageSize::Size2M,
                    Size1GiB::SIZE => crate::mem::PageSize::Size1G,
                    _ => unreachable!(),
                };

                Ok((address + offset as usize, property, size))
            }
            TranslateResult::NotMapped => {
                Err(Errno::InvArg.with_message("Translating unmapped page."))
            }
            TranslateResult::InvalidFrameAddress(_) => {
                Err(Errno::InvArg.with_message("Invalid frame address."))
            }
        }
    }

    fn update(
        &mut self,
        vaddr: VirtAddr,
        property: crate::mem::PageProperty,
    ) -> Result<crate::mem::PageSize> {
        let Ok((_, _, page_size)) = self.query(vaddr) else {
            return Err(Errno::InvArg.with_message("Updating unmapped page."));
        };

        let vaddr = Vaddr::new(page_size.align_down(vaddr) as u64);
        let property = kernel_property_converter(property);

        unsafe {
            match page_size {
                crate::mem::PageSize::Size4K => self
                    .update_property(Page::<Size4KiB>::containing_address(vaddr), property)
                    .map_err(|_| Errno::InvArg.with_message("Updating unmapped page."))?
                    .flush(),
                crate::mem::PageSize::Size2M => self
                    .update_property(Page::<Size2MiB>::containing_address(vaddr), property)
                    .map_err(|_| Errno::InvArg.with_message("Updating unmapped page."))?
                    .flush(),
                crate::mem::PageSize::Size1G => self
                    .update_property(Page::<Size1GiB>::containing_address(vaddr), property)
                    .map_err(|_| Errno::InvArg.with_message("Updating unmapped page."))?
                    .flush(),
            }
        }
        Ok(page_size)
    }

    fn deep_copy(&self) -> Arc<RwLock<dyn GeneralPageTable>> {
        let frame_allocator = &mut FRAME_ALLOCATOR.lock();

        let root_table_frame =
            <BitmapFrameAllocator as FrameAllocator<Size4KiB>>::allocate_frame(frame_allocator)
                .expect("Failed to allocate frame for root page table")
                .start_address();

        let target_root_vaddr =
            Vaddr::new(phys_to_virt(root_table_frame.as_u64() as PhysAddr) as u64);
        let root_table: &mut PageTable = unsafe { &mut *target_root_vaddr.as_mut_ptr() };
        root_table.zero();

        let mut stack: Vec<(*const PageTable, *mut PageTable, u8)> = alloc::vec![(
            self.lower_half_page_table() as *const _,
            root_table as *mut _,
            4
        )];

        while let Some((source_table, target_table, level)) = stack.pop() {
            for (index, entry) in (unsafe { &*source_table })
                .iter()
                .enumerate()
                .filter(|(_, entry)| !entry.is_unused())
            {
                if level == 1 || entry.flags().contains(PageTableFlags::HUGE_PAGE) {
                    let flags = entry.flags();
                    unsafe {
                        (&mut *target_table)[index].set_addr(entry.addr(), flags);
                    }
                } else {
                    let target_child_frame =
                        <BitmapFrameAllocator as FrameAllocator<Size4KiB>>::allocate_frame(
                            frame_allocator,
                        )
                        .expect("Failed to allocate frame for child page table")
                        .start_address();

                    let target_child_vaddr =
                        Vaddr::new(phys_to_virt(target_child_frame.as_u64() as PhysAddr) as u64);
                    let target_child_table =
                        unsafe { &mut *target_child_vaddr.as_mut_ptr::<PageTable>() };
                    target_child_table.zero();

                    unsafe {
                        (&mut *target_table)[index].set_addr(target_child_frame, entry.flags());
                    }

                    let source_child_vaddr =
                        Vaddr::new(phys_to_virt(entry.addr().as_u64() as PhysAddr) as u64);
                    stack.push((source_child_vaddr.as_ptr(), target_child_table, level - 1));
                }
            }
        }

        let page_table = unsafe {
            let kernel_page_table_ptr =
                Vaddr::new(self.higher_half_page_table() as *const _ as u64).as_mut_ptr();
            OffsetPageTable::new(
                &mut *kernel_page_table_ptr,
                root_table,
                phys_to_virt(0) as u64,
            )
        };
        Arc::new(RwLock::new(page_table))
    }

    fn activate(&self) {
        let lower_half = virt_to_phys(self.lower_half_page_table() as *const _ as VirtAddr);
        PgdLow.write(lower_half as u64);

        let higher_half = virt_to_phys(self.higher_half_page_table() as *const _ as VirtAddr);
        PgdHigh.write(higher_half as u64);
    }
}

unsafe impl FrameAllocator<Size4KiB> for BitmapFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        self.allocate_frames(1)
            .map(|addr| PhysFrame::containing_address(Paddr::new(addr as u64)))
    }
}

impl FrameDeallocator<Size4KiB> for BitmapFrameAllocator {
    unsafe fn deallocate_frame(&mut self, frame: PhysFrame<Size4KiB>) {
        self.deallocate_frames(frame.start_address().into(), 1);
    }
}
