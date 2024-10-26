use alloc::string::String;

use crate::fs::Inode;

pub struct NullDevice {
    path: String,
}

impl NullDevice {
    pub fn new() -> Self {
        Self {
            path: String::new(),
        }
    }
}

impl Inode for NullDevice {
    fn when_mounted(&mut self, path: String, _father: Option<crate::fs::InodeRef>) {
        self.path = path;
    }

    fn when_umounted(&mut self) {
        self.path = String::new();
    }

    fn get_path(&self) -> String {
        self.path.clone()
    }

    fn inode_type(&self) -> crate::fs::InodeTy {
        crate::fs::InodeTy::CharDevice
    }
}

