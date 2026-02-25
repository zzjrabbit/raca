use core::ops::Range;

use crate::object::KObjectBase;
use crate::{Errno, Result, impl_kobj, new_kobj};
use alloc::{sync::Arc, vec::Vec};
use kernel_hal::mem::{
    KERNEL_ASPACE_BASE, KERNEL_ASPACE_SIZE, MMUFlags, PageProperty, VirtAddr, VmSpace,
};
use kernel_hal::mem::{USER_ASPACE_BASE, USER_ASPACE_SIZE};
use spin::{Lazy, Mutex, RwLock};

use mapping::VmMapping;

use super::{PAGE_SIZE, Vmo, align_down_by_page_size, align_up_by_page_size};

mod mapping;
mod pf;
mod rw;

#[derive(Debug)]
pub struct Vmar {
    vm_space: Arc<VmSpace>,
    inner: RwLock<VmarInner>,
    lock: Mutex<()>,
    base_addr: VirtAddr,
    size: usize,
    is_root: bool,
    base: KObjectBase,
}

impl_kobj!(Vmar);

#[derive(Debug)]
struct VmarInner {
    vm_mappings: Vec<VmMapping>,
    children: Vec<Arc<Vmar>>,
}

impl Vmar {
    #[cfg(not(feature = "libos"))]
    pub fn new_root() -> Arc<Self> {
        Vmar::new_root_impl()
    }

    #[cfg(feature = "libos")]
    pub fn new_root() -> Arc<Self> {
        static ROOT: Lazy<Arc<Vmar>> = Lazy::new(Vmar::new_root_impl);
        ROOT.clone()
    }

    fn new_root_impl() -> Arc<Self> {
        new_kobj!({
            vm_space: Arc::new(VmSpace::new_user()),
            inner: RwLock::new(VmarInner {
                vm_mappings: Vec::new(),
                children: Vec::new(),
            }),
            lock: Mutex::new(()),
            base_addr: USER_ASPACE_BASE,
            size: USER_ASPACE_SIZE,
            is_root: true,
        })
    }

    pub fn kernel() -> Arc<Self> {
        static KERNEL: Lazy<Arc<Vmar>> = Lazy::new(|| {
            Arc::new(Vmar {
                vm_space: unsafe { VmSpace::kernel() },
                inner: RwLock::new(VmarInner {
                    vm_mappings: Vec::new(),
                    children: Vec::new(),
                }),
                lock: Mutex::new(()),
                base_addr: KERNEL_ASPACE_BASE,
                size: KERNEL_ASPACE_SIZE,
                is_root: true,
                base: KObjectBase::default(),
            })
        });
        KERNEL.clone()
    }
}

impl Vmar {
    pub fn activate(&self) {
        self.vm_space.activate();
    }

    pub fn base(&self) -> VirtAddr {
        self.base_addr
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn end(&self) -> VirtAddr {
        self.base() + self.size
    }

    pub fn page_count(&self) -> usize {
        self.size / PAGE_SIZE
    }
}

impl Vmar {
    fn add_child(&self, base: VirtAddr, size: usize) -> Result<Arc<Self>> {
        log::debug!("adding child: base={:#x} size={:#x}", base, size);
        let child = new_kobj!({
            vm_space: self.vm_space.clone(),
            inner: RwLock::new(VmarInner {
                vm_mappings: Vec::new(),
                children: Vec::new(),
            }),
            lock: Mutex::new(()),
            base_addr: base,
            size,
            is_root: false,
        });

        self.inner.write().children.push(child.clone());
        Ok(child)
    }

    pub fn create_child(&self, base: VirtAddr, size: usize) -> Result<Arc<Self>> {
        if !self.contains_range(base, size) || !size.is_multiple_of(PAGE_SIZE) {
            return Err(Errno::InvArg.no_message());
        }

        let _guard = self.lock.lock();
        if !self.lock.is_locked() {
            panic!("Lock optimized");
        }

        if !self.range_is_completely_free(base, size) {
            return Err(Errno::OutOfMemory.no_message());
        }

        self.add_child(base, size)
    }

    pub fn allocate_child(&self, size: usize) -> Result<Arc<Self>> {
        if size == 0 || !size.is_multiple_of(PAGE_SIZE) {
            return Err(Errno::InvArg.no_message());
        }

        if size > self.size {
            return Err(Errno::OutOfMemory.no_message());
        }

        let _guard = self.lock.lock();
        if !self.lock.is_locked() {
            panic!("Lock optimized");
        }

        let mut regions = {
            let inner = self.inner.read();
            inner
                .vm_mappings
                .iter()
                .map(|mapping| (mapping.start(), mapping.end()))
                .chain(
                    inner
                        .children
                        .iter()
                        .map(|child| (child.base(), child.end())),
                )
                .collect::<Vec<_>>()
        };
        regions.sort();

        let mut last_end = self.base();

        for (start, end) in regions {
            let usable = start - last_end;
            if usable >= size {
                return self.add_child(last_end, size);
            }
            last_end = end;
        }

        if self.end() - last_end < size {
            Err(Errno::OutOfMemory.no_message())
        } else {
            self.add_child(last_end, size)
        }
    }
}

impl Vmar {
    pub fn direct_map(&self, offset: usize, vmo: &Vmo, prop: PageProperty) -> Result<()> {
        let addr = self.base() + offset;
        let size = vmo.len();

        let aligned = align_down_by_page_size(addr);
        let mut cursor = self.vm_space.cursor(aligned)?;

        for id in 0..size / PAGE_SIZE {
            let offset = id * PAGE_SIZE;
            cursor.map(&vmo.get_ram(offset)?.unwrap().1, prop)?;
        }

        Ok(())
    }

    pub fn map(
        &self,
        offset: usize,
        vmo: &Arc<Vmo>,
        prop: PageProperty,
        process_overlap: bool,
    ) -> Result<()> {
        let addr = self.base() + offset;
        let size = vmo.len();

        log::debug!(
            "Vmar::map: addr={:#x} size={:#x} flags={:x}",
            addr,
            size,
            prop.flags
        );

        if !self.contains_range(addr, size) {
            return Err(Errno::InvArg.with_message("Out of VMAR range!"));
        }
        if size == 0 {
            return Ok(());
        }

        let _guard = self.lock.lock();
        if !self.lock.is_locked() {
            panic!("Lock optimized");
        }

        if !self.range_is_child_free(addr, size) {
            return Err(Errno::OutOfMemory.no_message());
        }

        let aligned = align_down_by_page_size(addr);
        let vm_mapping = VmMapping::new(vmo.clone(), aligned, size, prop, prop.flags);

        if process_overlap {
            self.insert_truncate_others(vm_mapping)?;
        } else {
            self.insert(vm_mapping)?;
        }

        #[cfg(feature = "libos")]
        for id in 0..size / PAGE_SIZE {
            self.handle_page_fault(aligned + id * PAGE_SIZE, prop.flags)?;
        }

        Ok(())
    }

    pub fn unmap(&self, addr: VirtAddr, size: usize) -> Result<()> {
        if size == 0 {
            return Ok(());
        }
        if !self.contains_range(addr, size) || !size.is_multiple_of(PAGE_SIZE) {
            return Err(Errno::InvArg.no_message());
        }

        let _guard = self.lock.lock();
        if !self.lock.is_locked() {
            panic!("Lock optimized");
        }

        if !self.range_is_child_free(addr, size) {
            return Err(Errno::InvArg.with_message("Range is in child vmar."));
        }

        let mut inner = self.inner.write();

        for mapping in inner.vm_mappings.iter_mut() {
            if mapping.contains_range(addr, size) {
                let aligned = align_down_by_page_size(addr);
                let size = align_up_by_page_size(size + addr - aligned);

                let mut cursor = self.vm_space.cursor(aligned)?;
                cursor.unmap(size)?;
            }
        }

        inner
            .vm_mappings
            .retain(|mapping| !mapping.contains_range(addr, size));

        Ok(())
    }

    pub fn protect(&self, addr: VirtAddr, size: usize, flags: MMUFlags) -> Result<()> {
        if size == 0 {
            return Ok(());
        }
        if !self.contains_range(addr, size) || !size.is_multiple_of(PAGE_SIZE) {
            return Err(Errno::InvArg.with_message("Size is not a multiple of page size."));
        }

        let _guard = self.lock.lock();
        if !self.lock.is_locked() {
            panic!("Lock optimized");
        }

        if !self.range_is_child_free(addr, size) {
            return Err(Errno::InvArg.with_message("Range is in child vmar."));
        }

        {
            let aligned = align_down_by_page_size(addr);
            let size = align_up_by_page_size(size + addr - aligned);

            let mut cursor = self.vm_space.cursor(aligned)?;
            if let Err(e) = cursor.protect(size, |cprop| {
                cprop.flags.insert(flags);
            }) && e.errno() != Errno::NotMapped
            {
                return Err(e);
            }
        }

        let mappings = self
            .inner
            .read()
            .vm_mappings
            .iter()
            .filter(|mapping| mapping.contains_range(addr, size))
            .map(|mapping| mapping.start())
            .collect::<Vec<_>>();

        for &vm_mapping_addr in mappings.iter() {
            let mapping = self.remove_by_addr(vm_mapping_addr).unwrap();

            if mapping.perm().contains(flags) {
                continue;
            }

            let split_range =
                get_intersected_range(&(mapping.start()..mapping.end()), &(addr..addr + size));
            let split_left = split_range.start;
            let split_right = split_range.end;

            let (left, mut taken, right) = mapping.split_range(split_left, split_right)?;

            if let Some(left) = left {
                self.insert(left)?;
            }
            if let Some(right) = right {
                self.insert(right)?;
            }

            let mut prop = taken.prop();
            prop.flags |= flags;
            taken.set_prop(prop);

            taken.set_perm(taken.perm() | flags);

            self.insert(taken)?;
        }

        Ok(())
    }

    pub fn query(&self, addr: VirtAddr) -> usize {
        /*if let Some(child) = self.find_child(addr) {
            return child.query(addr);
        }
        let vmo = self
            .inner
            .read()
            .vm_mappings
            .iter()
            .find(|mapping| mapping.contains(addr))?
            .vmo()
            .clone();
        Some(vmo)*/
        self.vm_space
            .cursor(align_down_by_page_size(addr))
            .unwrap()
            .query()
            .unwrap()
            .0
            .start()
    }
}

impl Vmar {
    fn remove_by_addr(&self, addr: VirtAddr) -> Option<VmMapping> {
        let index = self
            .inner
            .read()
            .vm_mappings
            .iter()
            .position(|mapping| mapping.start() == addr)?;
        let mapping = self.inner.write().vm_mappings.remove(index);
        Some(mapping)
    }

    fn insert(&self, mapping: VmMapping) -> Result<()> {
        let mut inner = self.inner.write();
        inner.vm_mappings.push(mapping);
        Ok(())
    }

    fn insert_truncate_others(&self, mapping: VmMapping) -> Result<()> {
        let mappings_to_remove = self
            .inner
            .read()
            .vm_mappings
            .iter()
            .filter(|other| other.overlaps(&mapping))
            .map(|mapping| mapping.start())
            .collect::<Vec<_>>();

        for addr in mappings_to_remove {
            let vm_mapping = self.remove_by_addr(addr).unwrap();

            let vm_mapping_range = vm_mapping.start()..vm_mapping.end();
            let required_range = mapping.start()..mapping.end();

            let split_range = get_intersected_range(&vm_mapping_range, &required_range);
            let split_left = split_range.start;
            let split_right = split_range.end;

            let (left, _taken, right) = vm_mapping.split_range(split_left, split_right)?;
            if let Some(left) = left {
                self.insert(left)?;
            }
            if let Some(right) = right {
                self.insert(right)?;
            }
        }

        self.insert(mapping)?;
        Ok(())
    }
}

impl Vmar {
    pub fn deep_clone(&self) -> Result<Arc<Self>> {
        if !self.is_root {
            return Err(Errno::InvArg.with_message("Cannot deep clone non-root VMAR!"));
        }

        let _guard = self.lock.lock();
        if !self.lock.is_locked() {
            panic!("Lock optimized");
        }

        let mut vm_mappings = Vec::new();
        for mapping in self.inner.write().vm_mappings.iter_mut() {
            vm_mappings.push(mapping.clone()?);
            if mapping.perm().contains(MMUFlags::WRITE) {
                let address = mapping.start();
                let size = mapping.size();

                let mut cursor = self.vm_space.cursor(address)?;
                cursor.protect(size, |cprop| {
                    cprop.flags.remove(MMUFlags::WRITE);
                })?;
            }
        }

        Ok(new_kobj!({
            vm_space: Arc::new(VmSpace::new_user()),
            inner: RwLock::new(VmarInner {
                vm_mappings,
                children: Vec::new(),
            }),
            lock: Mutex::new(()),
            base_addr: self.base(),
            size: self.size,
            is_root: true,
        }))
    }
}

impl Vmar {
    fn contains(&self, address: VirtAddr) -> bool {
        address >= self.base() && address < self.end()
    }

    pub fn contains_range(&self, address: VirtAddr, size: usize) -> bool {
        address >= self.base() && address + size <= self.end()
    }

    fn overlap_range(&self, start: VirtAddr, size: usize) -> bool {
        !(start >= self.end() || start + size <= self.base())
    }

    fn range_is_child_free(&self, start: VirtAddr, size: usize) -> bool {
        !self
            .inner
            .read()
            .children
            .iter()
            .any(|child| child.overlap_range(start, size))
    }

    fn range_is_completely_free(&self, start: VirtAddr, size: usize) -> bool {
        self.range_is_child_free(start, size)
            && !self
                .inner
                .read()
                .vm_mappings
                .iter()
                .any(|mapping| mapping.overlap_range(start, size))
    }

    fn find_child(&self, address: VirtAddr) -> Option<Arc<Vmar>> {
        self.inner
            .read()
            .children
            .iter()
            .find(|child| child.contains(address))
            .cloned()
    }
}

fn get_intersected_range(range1: &Range<VirtAddr>, range2: &Range<VirtAddr>) -> Range<VirtAddr> {
    range1.start.max(range2.start)..range1.end.min(range2.end)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_vmar() {
        let _vmar = Vmar::new_root();
    }

    #[test]
    fn maps() {
        let vmar = Vmar::new_root();
        vmar.activate();

        let child = vmar.allocate_child(4 * 1024).unwrap();
        child
            .map(
                0,
                &Vmo::allocate_ram(child.page_count()).unwrap(),
                PageProperty::kernel_code(),
                true,
            )
            .unwrap();
        let address = child.base();

        child.protect(address, 4 * 1024, MMUFlags::WRITE).unwrap();

        child.unmap(address, 4 * 1024).unwrap();
    }

    #[test]
    fn read_write() {
        let vmar = Vmar::new_root();
        vmar.activate();

        let child = vmar.allocate_child(4 * 1024).unwrap();
        child
            .map(
                0,
                &Vmo::allocate_ram(child.page_count()).unwrap(),
                PageProperty::kernel_code(),
                true,
            )
            .unwrap();
        let address = child.base();

        child.protect(address, 4 * 1024, MMUFlags::WRITE).unwrap();

        child.write_val(address, &42usize).unwrap();
        assert_eq!(child.read_val::<usize>(address).unwrap(), 42);

        child.unmap(address, 4 * 1024).unwrap();
    }

    #[test]
    fn read_direct() {
        let vmar = Vmar::new_root();
        vmar.activate();

        let child = vmar.allocate_child(4 * 1024).unwrap();
        child
            .map(
                0,
                &Vmo::allocate_ram(child.page_count()).unwrap(),
                PageProperty::user_code(),
                true,
            )
            .unwrap();
        let address = child.base();
        log::debug!("address: {:#x}", address);

        child.protect(address, 4 * 1024, MMUFlags::WRITE).unwrap();

        child.write_val(address, &42usize).unwrap();

        let ptr = address as *const usize;

        assert_eq!(unsafe { ptr.read() }, 42);

        child.unmap(address, 4 * 1024).unwrap();
    }
}
