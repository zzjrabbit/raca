use core::{fmt::Debug, ops::{Deref, DerefMut}};

use alloc::{
    collections::btree_map::BTreeMap,
    string::{String, ToString},
    sync::Arc, vec::Vec,
};
use spin::RwLock;

use crate::fs::dev::NullDevice;

use super::{Inode, InodeType, Path};

#[derive(Clone)]
pub struct FileRef(Arc<RwLock<File>>);

impl FileRef {
    pub fn new(inode: Inode, file_type: FileType, name: String, path: Path) -> Self {
        let file = File {
            inode,
            file_type,
            name,
            path,
            children: BTreeMap::new(),
        };
        Self(Arc::new(RwLock::new(file)))
    }

    pub fn create_dir(&self, relative_path: Path) -> FileRef {
        let relative_path = {
            let mut tmp = relative_path.clone();
            if tmp.ends_with("/") {
                tmp.pop();
            }
            tmp
        };

        if relative_path.is_empty() {
            return self.clone();
        }

        let parents = relative_path.split("/").collect::<Vec<_>>();

        let mut current = self.clone();

        let mut current_relative_path = Path::new("/");

        for parent in parents {
            let child = current.clone().read().get_child(parent).clone();
            if let Some(child) = child {
                current = child;
            } else {
                
                current_relative_path += parent;

                let child_dir = FileRef::new(
                    Inode::new(NullDevice, InodeType::Dir),
                    FileType::Dir,
                    parent.to_string(),
                    current_relative_path.clone(),
                );
                current
                    .write()
                    .children
                    .insert(parent.to_string(), child_dir.clone());
                current = child_dir;
                current_relative_path += "/";
            }
        }

        current.clone()
    }
}

impl Deref for FileRef {
    type Target = Arc<RwLock<File>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for FileRef {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Debug for FileRef {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("FileRef")
            .field("name", &self.read().get_file_name())
            .field("path", &self.read().get_file_path())
            .finish()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum FileType {
    Fifo = 1,
    CharDevice = 2,
    BlockDevice = 3,
    Socket = 4,
    Dir = 5,
    File = 6,
    SymLink = 7,
}

pub struct File {
    inode: Inode,
    file_type: FileType,
    name: String,
    path: Path,
    children: BTreeMap<String, FileRef>,
}

impl File {
    pub fn get_file_type(&self) -> FileType {
        self.file_type
    }

    pub fn get_file_name(&self) -> &str {
        &self.name
    }

    pub fn get_file_path(&self) -> Path {
        self.path.clone()
    }

    pub fn add_child(&mut self, child: FileRef) {
        self.children
            .insert(child.read().get_file_name().into(), child.clone());
    }

    pub fn remove_child(&mut self, child: FileRef) {
        self.children.remove(child.read().get_file_name());
    }

    pub fn get_child(&self, name: &str) -> Option<FileRef> {
        self.children.get(name).cloned()
    }

    pub fn get_children(&self) -> BTreeMap<String, FileRef> {
        self.children.clone()
    }
}

impl Deref for File {
    type Target = Inode;

    fn deref(&self) -> &Self::Target {
        &self.inode
    }
}

impl DerefMut for File {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inode
    }
}
