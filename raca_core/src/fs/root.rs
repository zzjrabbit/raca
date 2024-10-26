use alloc::{collections::BTreeMap, string::String};

use super::{Inode, InodeRef};

pub struct RootFS {
    nodes: BTreeMap<String, InodeRef>,
    path: String,
}

impl RootFS {
    pub const fn new() -> Self {
        Self {
            nodes: BTreeMap::new(),
            path: String::new(),
        }
    }
}

impl Inode for RootFS {
    fn when_mounted(&mut self, path: String, _father: Option<InodeRef>) {
        self.path = path;
    }

    fn when_umounted(&mut self) {
        for (_, node) in self.nodes.iter() {
            node.write().when_umounted();
        }
    }

    fn mount(&mut self, node: InodeRef, name: String) {
        self.nodes.insert(name, node);
    }

    fn get_path(&self) -> String {
        self.path.clone()
    }
}
