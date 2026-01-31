#![allow(dead_code)]

use crate::{Errno, Result, mem::PAGE_SIZE};
use alloc::{sync::Arc, vec::Vec};
use kernel_hal::{
    io::IoMem,
    mem::{PhysicalMemory, PhysicalMemoryAllocOptions, VirtAddr},
};
use spin::RwLock;

mod rw;

#[derive(Debug, Clone)]
pub struct Vmo {
    inner: Arc<VmoInner>,
}

type PhysicalMemoryRef = Arc<PhysicalMemory>;

#[derive(Debug)]
enum VmoInner {
    Ram {
        frames: RwLock<Vec<Option<PhysicalMemoryRef>>>,
    },
    IoMem {
        iomem: Arc<IoMem>,
        offset: usize,
    },
}

impl Vmo {
    pub fn allocate_ram(count: usize) -> Result<Self> {
        Ok(Self {
            inner: Arc::new(VmoInner::Ram {
                frames: RwLock::new(alloc::vec![None; count]),
            }),
        })
    }

    pub fn acquire_iomem(address: VirtAddr, length: usize) -> Result<Self> {
        Ok(Self {
            inner: Arc::new(VmoInner::IoMem {
                iomem: IoMem::acquire(address..address + length)?,
                offset: address % PAGE_SIZE,
            }),
        })
    }

    pub fn deep_clone(&self) -> Result<Self> {
        match self.inner.as_ref() {
            VmoInner::Ram { frames } => {
                let mut new_frames = alloc::vec![None; frames.read().len()];
                for (i, dest) in new_frames.iter_mut().enumerate() {
                    let source = frames.read()[i].clone();
                    if let Some(source) = source {
                        let frame = Arc::new(PhysicalMemoryAllocOptions::new().allocate()?);

                        let mut buffer = alloc::vec![0u8; PAGE_SIZE];
                        source.read_bytes(0, &mut buffer)?;
                        frame.write_bytes(0, &buffer)?;

                        *dest = Some(frame.clone());
                    }
                }
                Ok(Self {
                    inner: Arc::new(VmoInner::Ram {
                        frames: RwLock::new(new_frames),
                    }),
                })
            }
            VmoInner::IoMem { .. } => {
                Err(Errno::AccessDenied.with_message("Attempting to deep clone IoMem."))
            }
        }
    }
}

impl Vmo {
    pub(super) fn get_ram(&self, offset: usize) -> Result<Option<(usize, PhysicalMemoryRef)>> {
        match self.inner.as_ref() {
            VmoInner::Ram { frames } => {
                let id = offset / PAGE_SIZE;
                let page_offset = offset % PAGE_SIZE;

                let frame = frames.read()[id].clone();
                match frame {
                    Some(frame) => Ok(Some((page_offset, frame))),
                    None => {
                        let frame = Arc::new(PhysicalMemoryAllocOptions::new().allocate()?);
                        frames.write()[id] = Some(frame.clone());
                        Ok(Some((page_offset, frame)))
                    }
                }
            }
            VmoInner::IoMem { .. } => Ok(None),
        }
    }

    pub(super) fn get_iomem(&self) -> Option<(Arc<IoMem>, usize)> {
        match self.inner.as_ref() {
            VmoInner::Ram { .. } => None,
            VmoInner::IoMem { iomem, offset } => Some((iomem.clone(), *offset)),
        }
    }

    pub(super) fn commited(&self, id: usize) -> bool {
        match self.inner.as_ref() {
            VmoInner::Ram { frames } => frames.read()[id].is_some(),
            VmoInner::IoMem { .. } => true,
        }
    }
}

impl Vmo {
    pub fn len(&self) -> usize {
        match self.inner.as_ref() {
            VmoInner::Ram { frames } => frames.read().len() * PAGE_SIZE,
            VmoInner::IoMem { iomem, .. } => iomem.size(),
        }
    }

    pub fn is_empty(&self) -> bool {
        match self.inner.as_ref() {
            VmoInner::Ram { frames } => frames.read().is_empty(),
            VmoInner::IoMem { iomem, .. } => iomem.size() == 0,
        }
    }

    pub fn is_iomem(&self) -> bool {
        match self.inner.as_ref() {
            VmoInner::Ram { .. } => false,
            VmoInner::IoMem { .. } => true,
        }
    }
}

impl Vmo {
    pub fn split(&self, id: usize) -> Result<Self> {
        match self.inner.as_ref() {
            VmoInner::Ram { frames } => {
                let mut frames = frames.write();
                let new_frames = frames.split_off(id);
                Ok(Self {
                    inner: Arc::new(VmoInner::Ram {
                        frames: RwLock::new(new_frames),
                    }),
                })
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
