use core::{mem::{transmute, variant_count}, sync::atomic::{AtomicUsize, Ordering}};

use alloc::collections::btree_map::BTreeMap;
use spin::Mutex;

use crate::error::{RcError, RcResult};

use super::{FileRef, FileType, Path, ROOT};

mod kernel;
mod user;

pub use kernel::*;
pub use user::*;

pub enum OpenMode {
    Read = 0,
    Write = 1,
    ReadWrite = 2,
}

impl OpenMode {
    pub fn from_usize(mode: usize) -> RcResult<Self> {
        let length = variant_count::<Self>();
        if mode >= length {
            Err(RcError::INVALID_ARGS)
        } else {
            Ok(unsafe { transmute(mode as u8) })
        }
    }
}

pub type FileDescriptor = usize;
type FileTuple = (FileRef, OpenMode, usize);

pub struct FileDescriptorManager {
    file_descriptors: BTreeMap<FileDescriptor, FileTuple>,
    file_descriptor_allocator: AtomicUsize,
    cwd: Mutex<FileRef>,
}

impl FileDescriptorManager {
    pub fn new(file_descriptors: BTreeMap<FileDescriptor, FileTuple>) -> Self {
        Self {
            file_descriptors,
            file_descriptor_allocator: AtomicUsize::new(0), 
            cwd: Mutex::new(ROOT.clone()),
        }
    }

    pub fn get_new_fd(&self) -> FileDescriptor {
        self.file_descriptor_allocator
            .fetch_add(1, Ordering::Relaxed)
    }

    pub fn add_file(&mut self, file: FileRef, mode: OpenMode) -> FileDescriptor {
        let new_fd = self.get_new_fd();
        self
            .file_descriptors
            .insert(new_fd, (file, mode, 0));
        new_fd
    }

    pub fn change_cwd(&self, path: Path) {
        if let Some(file) = get_file_by_path(path) {
            if file.read().get_file_type() == FileType::Dir {
                *self.cwd.lock() = file;
            }
        }
    }

    pub fn get_cwd(&self) -> Path {
        self.cwd.lock().read().get_file_path()
    }
}

fn get_file_by_path(path: Path) -> Option<FileRef> {
    let root = ROOT.clone();

    let path = path.split("/");

    let mut node = root;

    for path_node in path {
        if path_node.len() > 0 {
            let child = if let Some(child) = node.read().get_child(path_node) {
                child
            } else {
                return None;
            };
            core::mem::drop(core::mem::replace(&mut node, child));
        }
    }

    Some(node.clone())
}
