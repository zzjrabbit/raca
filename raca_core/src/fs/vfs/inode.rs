use core::ops::{Deref, DerefMut};

use alloc::sync::Arc;
use spin::RwLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum InodeType {
    Dir = 1,
    File = 2,
}

pub trait InodeFunction {
    fn read_at(&self, offset: usize, data: &mut [u8]) -> usize;
    fn write_at(&mut self, offset: usize, data: &[u8]) -> usize;

    fn len(&self) -> usize;
}

#[derive(Clone)]
pub struct Inode {
    func: Arc<RwLock<dyn InodeFunction + Send + Sync>>,
    inode_type: InodeType,
}

impl Inode {
    pub fn new<T>(func: T, inode_type: InodeType) -> Self
    where
        T: InodeFunction + Send + Sync + 'static,
    {
        Self {
            func: Arc::new(RwLock::new(func)),
            inode_type,
        }
    }

    pub fn get_inode_type(&self) -> InodeType {
        self.inode_type
    }

    pub fn read_at(&self, offset: usize, data: &mut [u8]) -> usize {
        if self.inode_type == InodeType::File {
            self.func.read().read_at(offset, data)
        } else {
            0
        }
    }

    pub fn write_at(&self, offset: usize, data: &[u8]) -> usize {
        if self.inode_type == InodeType::File {
            self.func.write().write_at(offset, data)
        } else {
            0
        }
    }

    pub fn len(&self) -> usize {
        if self.inode_type == InodeType::File {
            self.func.read().len()
        } else {
            0
        }
    }
}

impl Deref for Inode {
    type Target = Arc<RwLock<dyn InodeFunction + Send + Sync>>;
    fn deref(&self) -> &Self::Target {
        &self.func
    }
}

impl DerefMut for Inode {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.func
    }
}
