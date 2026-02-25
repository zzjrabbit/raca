#![allow(dead_code)]

use crate::{Errno, Result, impl_kobj, mem::PAGE_SIZE, new_kobj, object::KObjectBase};
use alloc::{collections::btree_map::BTreeMap, sync::Arc};
use kernel_hal::{
    io::IoMem,
    mem::{PhysicalMemory, PhysicalMemoryAllocOptions, VirtAddr},
};
use spin::RwLock;

mod rw;

#[derive(Debug)]
pub struct Vmo {
    inner: VmoInner,
    base: KObjectBase,
}

impl_kobj!(Vmo);

type PhysicalMemoryRef = Arc<PhysicalMemory>;

#[derive(Debug)]
enum VmoInner {
    Ram {
        frames: RwLock<BTreeMap<usize, PhysicalMemoryRef>>,
        count: usize,
    },
    IoMem {
        iomem: Arc<IoMem>,
        offset: usize,
    },
}

impl Vmo {
    pub fn allocate_ram(count: usize) -> Result<Arc<Self>> {
        Ok(new_kobj!({
            inner: VmoInner::Ram {
                frames: RwLock::new(BTreeMap::new()),
                count,
            },
        }))
    }

    pub fn acquire_iomem(address: VirtAddr, length: usize) -> Result<Arc<Self>> {
        Ok(new_kobj!({
            inner: VmoInner::IoMem {
                iomem: IoMem::acquire(address..address + length)?,
                offset: address % PAGE_SIZE,
            },
        }))
    }

    pub fn deep_clone(&self) -> Result<Arc<Self>> {
        match &self.inner {
            VmoInner::Ram { frames, count } => {
                let mut new_frames = BTreeMap::new();
                for (&i, source) in frames.read().iter() {
                    let dest = Arc::new(PhysicalMemoryAllocOptions::new().allocate()?);

                    let mut buffer = alloc::vec![0u8; PAGE_SIZE];
                    source.read_bytes(0, &mut buffer)?;
                    dest.write_bytes(0, &buffer)?;

                    new_frames.insert(i, dest);
                }
                Ok(new_kobj!({
                    inner: VmoInner::Ram {
                        frames: RwLock::new(new_frames),
                        count: *count,
                    },
                }))
            }
            VmoInner::IoMem { .. } => {
                Err(Errno::AccessDenied.with_message("Attempting to deep clone IoMem."))
            }
        }
    }
}

impl Vmo {
    pub(super) fn get_ram(&self, offset: usize) -> Result<Option<(usize, PhysicalMemoryRef)>> {
        match &self.inner {
            VmoInner::Ram { frames, count } => {
                let id = offset / PAGE_SIZE;
                let page_offset = offset % PAGE_SIZE;

                if id >= *count {
                    return Err(Errno::InvArg.with_message("Offset out of bounds"));
                }

                let frame = frames.read().get(&id).cloned();
                match frame {
                    Some(frame) => Ok(Some((page_offset, frame))),
                    None => {
                        let frame = Arc::new(PhysicalMemoryAllocOptions::new().allocate()?);
                        frames.write().insert(id, frame.clone());
                        frame.zero()?;
                        Ok(Some((page_offset, frame)))
                    }
                }
            }
            VmoInner::IoMem { .. } => Ok(None),
        }
    }

    pub(super) fn get_iomem(&self) -> Option<(Arc<IoMem>, usize)> {
        match &self.inner {
            VmoInner::Ram { .. } => None,
            VmoInner::IoMem { iomem, offset } => Some((iomem.clone(), *offset)),
        }
    }

    pub(super) fn commited(&self, id: usize) -> bool {
        match &self.inner {
            VmoInner::Ram { frames, .. } => frames.read().contains_key(&id),
            VmoInner::IoMem { .. } => true,
        }
    }
}

impl Vmo {
    /// Returns the length of the VMO in bytes.
    pub fn len(&self) -> usize {
        match &self.inner {
            VmoInner::Ram { frames: _, count } => count * PAGE_SIZE,
            VmoInner::IoMem { iomem, .. } => iomem.size(),
        }
    }

    pub fn is_empty(&self) -> bool {
        match &self.inner {
            VmoInner::Ram { frames: _, count } => *count == 0,
            VmoInner::IoMem { iomem, .. } => iomem.size() == 0,
        }
    }

    pub fn is_iomem(&self) -> bool {
        match &self.inner {
            VmoInner::Ram { .. } => false,
            VmoInner::IoMem { .. } => true,
        }
    }
}

impl Vmo {
    pub fn split(&self, id: usize) -> Result<Arc<Self>> {
        match &self.inner {
            VmoInner::Ram { frames, count } => {
                let mut frames = frames.write();
                let new_frames = frames.split_off(&id);
                Ok(new_kobj!({
                    inner: VmoInner::Ram {
                        frames: RwLock::new(new_frames),
                        count: count - id,
                    },
                }))
            }
            VmoInner::IoMem { .. } => Err(Errno::InvArg.no_message()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_vmo() {
        let _vmo = Vmo::allocate_ram(100).unwrap();
        let _vmo = Vmo::acquire_iomem(0x1000000, 4096).unwrap();
    }

    #[test]
    fn vmo_read_write() {
        let vmo = Vmo::allocate_ram(1).unwrap();
        vmo.write_val(100, &42usize).unwrap();
        assert_eq!(vmo.read_val::<usize>(100).unwrap(), 42);
    }

    #[test]
    fn vmo_split() {
        let vmo = Vmo::allocate_ram(10).unwrap();
        let vmo1 = vmo.split(5).unwrap();
        assert_eq!(vmo.len(), 5 * PAGE_SIZE);
        assert_eq!(vmo1.len(), 5 * PAGE_SIZE);
    }
}
