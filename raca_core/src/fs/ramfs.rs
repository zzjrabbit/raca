use alloc::{string::ToString, vec::Vec, vec};

use super::*;

pub fn create_ramfs_from_cpio<const BLOCK_SIZE: usize>(
    mut father_path: Path,
    cpio: &[u8],
) -> FileRef {
    father_path.delete_end_spliters();

    let root_inode = Inode::new(RamfsInode::<BLOCK_SIZE>::new_dir(), InodeType::Dir);
    let root = FileRef::new(
        root_inode,
        FileType::Dir,
        "".into(),
        father_path.dir_format(),
    );

    let files = cpio_reader::iter_files(cpio).collect::<Vec<_>>();

    for file in files {
        let path = father_path.join(Path::new(file.name().to_string()));
        let name = path.name();
        let parent_dir = path.parent();

        let dir = match parent_dir {
            Some(parent) => root.create_dir(parent.clone()),
            None => root.clone(),
        };

        let inode = Inode::new(
            RamfsInode::<BLOCK_SIZE>::new_file(file.file()),
            InodeType::File,
        );
        let file = FileRef::new(inode, FileType::File, name, path);
        dir.write().add_child(file);
    }

    root.clone()
}

pub struct RamfsInode<const BLOCK_SIZE: usize> {
    data: Vec<&'static mut [u8]>,
}

impl<const BLOCK_SIZE: usize> RamfsInode<BLOCK_SIZE> {
    pub fn new_dir() -> Self {
        Self { data: Vec::new() }
    }

    pub fn new_file(data: &[u8]) -> Self {
        if data.len() == 0 {
            return Self::new_dir();
        }

        let block_number = (data.len() + BLOCK_SIZE - 1) / BLOCK_SIZE;
        let mut blocks = Vec::with_capacity(block_number);

        for i in 0..block_number - 1 {
            let block = vec![0;BLOCK_SIZE].leak();

            block.copy_from_slice(&data[i * BLOCK_SIZE..(i + 1) * BLOCK_SIZE]);
            blocks.push(block);
        }

        let block = vec![0;BLOCK_SIZE].leak();

        for i in (block_number-1)*BLOCK_SIZE .. data.len() {
            block[i - (block_number - 1) * BLOCK_SIZE] = data[i];
        }

        blocks.push(block);

        Self { data: blocks }
    }
}

impl<const BLOCK_SIZE: usize> InodeFunction for RamfsInode<BLOCK_SIZE> {
    fn read_at(&self, offset: usize, data: &mut [u8]) -> usize {
        if offset < data.len() * BLOCK_SIZE {
            let start = offset;
            let end = core::cmp::min(offset + data.len(), self.data.len() * BLOCK_SIZE);
            for idx in start..end {
                let block_id = idx / BLOCK_SIZE;
                let block_offset = idx % BLOCK_SIZE;
                data[idx - start] = self.data[block_id][block_offset];
            }
        }
        0
    }

    fn write_at(&mut self, _offset: usize, _data: &[u8]) -> usize {
        0
    }

    fn len(&self) -> usize {
        self.data.len() * BLOCK_SIZE
    }
}
