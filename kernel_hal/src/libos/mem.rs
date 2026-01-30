use std::{num::NonZeroUsize, os::fd::OwnedFd, ptr::NonNull, sync::Arc};

use nix::{
    fcntl::{self, OFlag},
    sys::{
        mman::{MapFlags, ProtFlags, mmap, mprotect, munmap},
        stat::Mode,
    },
    unistd,
};
use spin::{Lazy, RwLock};
use tempfile::tempdir;

use crate::{GeneralPageTable, MMUFlags, PageProperty, PageSize, PhysAddr, VirtAddr};

pub(super) const PMEM_MAP_VADDR: VirtAddr = 0x8_0000_0000;
pub(super) const PMEM_SIZE: usize = 0x4000_0000;

pub(super) static PHYS_MEM: Lazy<Memory> = Lazy::new(|| Memory::new(PMEM_SIZE));

pub fn phys_to_virt(phys: PhysAddr) -> VirtAddr {
    phys + PMEM_MAP_VADDR
}

pub fn virt_to_phys(virt: VirtAddr) -> PhysAddr {
    virt - PMEM_MAP_VADDR
}

pub fn kernel_page_table() -> LibOsPageTable {
    LibOsPageTable
}

pub struct LibOsPageTable;

impl GeneralPageTable for LibOsPageTable {
    fn activate(&self) {}

    fn deep_copy(&self) -> std::sync::Arc<spin::RwLock<dyn GeneralPageTable>> {
        Arc::new(RwLock::new(Self))
    }

    fn map(
        &mut self,
        page: crate::Page,
        paddr: PhysAddr,
        property: PageProperty,
    ) -> Result<(), errors::Error> {
        PHYS_MEM.mmap(page.vaddr, page.size as usize, paddr, property);
        Ok(())
    }

    fn unmap(&mut self, vaddr: VirtAddr) -> Result<PageSize, errors::Error> {
        PHYS_MEM.munmap(vaddr, PageSize::Size4K as usize);
        Ok(PageSize::Size4K)
    }

    fn update(
        &mut self,
        vaddr: VirtAddr,
        property: PageProperty,
    ) -> Result<PageSize, errors::Error> {
        PHYS_MEM.mprotect(vaddr, PageSize::Size4K as usize, property);
        Ok(PageSize::Size4K)
    }
}

pub struct Memory {
    size: usize,
    fd: OwnedFd,
}

impl Memory {
    pub fn new(size: usize) -> Self {
        let dir = tempdir().expect("Unable to get temp dir!");
        let file = dir.path().join("raca_libos_pemem");

        let fd = fcntl::open(
            &file,
            OFlag::O_CREAT | OFlag::O_EXCL | OFlag::O_RDWR,
            Mode::S_IRWXU,
        )
        .expect("Failed to open pmem file.");
        unistd::ftruncate(&fd, size as _).expect("Failed to set size of physical memory.");

        let mem = Self { size, fd };
        mem.mmap(
            PMEM_MAP_VADDR,
            size,
            0,
            PageProperty::new(
                MMUFlags::READ | MMUFlags::WRITE,
                Default::default(),
                Default::default(),
            ),
        );
        mem
    }

    pub fn mmap(&self, vaddr: VirtAddr, len: usize, paddr: PhysAddr, prop: PageProperty) {
        assert!(paddr < self.size);
        assert!(paddr + len <= self.size);

        let prot = ProtFlags::from(prop);
        let prot_no_exec = prot - ProtFlags::PROT_EXEC;
        let flags = MapFlags::MAP_SHARED | MapFlags::MAP_FIXED;
        let fd = &self.fd;
        let offset = paddr as _;

        unsafe {
            mmap(
                Some(NonZeroUsize::new(vaddr).expect("mapping at zero address.")),
                NonZeroUsize::new(len).expect("mapping for length 0."),
                prot_no_exec,
                flags,
                fd,
                offset,
            )
            .unwrap_or_else(|err| {
                panic!("Failed to map physical memory: {}", err);
            });
        }
        if prop.flags.contains(MMUFlags::EXECUTE) {
            self.mprotect(vaddr, len, prop);
        }
    }

    pub fn munmap(&self, vaddr: VirtAddr, len: usize) {
        unsafe {
            munmap(
                NonNull::new(vaddr as _).expect("Attempt to unmap zero address"),
                len,
            )
            .unwrap_or_else(|err| panic!("Failed to unmap: vaddr={:#x} {:?}", vaddr, err));
        }
    }

    pub fn mprotect(&self, vaddr: VirtAddr, len: usize, prop: PageProperty) {
        unsafe {
            mprotect(
                NonNull::new(vaddr as _).expect("Attempt to protect zero address"),
                len,
                ProtFlags::from(prop),
            )
            .unwrap_or_else(|err| panic!("Failed to protect: vaddr={:#x} {:?}", vaddr, err));
        }
    }
}

impl From<PageProperty> for ProtFlags {
    fn from(value: PageProperty) -> Self {
        let vflags = value.flags;
        let mut flags = Self::empty();
        if vflags.contains(MMUFlags::READ) {
            flags |= Self::PROT_READ;
        }
        if vflags.contains(MMUFlags::WRITE) {
            flags |= Self::PROT_WRITE;
        }
        if vflags.contains(MMUFlags::EXECUTE) {
            flags |= Self::PROT_EXEC;
        }
        flags
    }
}
