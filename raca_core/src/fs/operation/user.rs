use alloc::{boxed::Box, sync::Arc};
use spin::RwLock;

use crate::{
    error::{RcError, RcResult},
    fs::Path,
    task::{Process, SCHEDULER},
};

use super::{get_file_by_path, FileDescriptor, OpenMode};

fn get_current_process() -> Arc<RwLock<Box<Process>>> {
    SCHEDULER
        .lock()
        .current_thread()
        .upgrade()
        .unwrap()
        .read()
        .process
        .upgrade()
        .unwrap()
}

pub fn open(path: Path, mode: OpenMode) -> RcResult<FileDescriptor> {
    let current_process = get_current_process();

    let fd = current_process
        .write()
        .file_descriptor_manager
        .add_file(get_file_by_path(path).ok_or(RcError::NOT_FOUND)?, mode);

    Ok(fd)
}

pub fn read(fd: FileDescriptor, buf: &mut [u8]) -> RcResult<usize> {
    let current_process = get_current_process();
    let current_process = current_process.read();

    if let Some((inode, mode, offset)) = current_process
        .file_descriptor_manager
        .file_descriptors
        .get(&fd)
    {
        match mode {
            OpenMode::Read | OpenMode::ReadWrite => Ok(inode.read().read_at(*offset, buf)),

            _ => Err(RcError::ACCESS_DENIED),
        }
    } else {
        Err(RcError::INVALID_ARGS)
    }
}

pub fn write(fd: FileDescriptor, buf: &[u8]) -> RcResult<usize> {
    let current_process = get_current_process();
    let current_process = current_process.read();
    let current_file_descriptor_manager = &current_process.file_descriptor_manager;

    if let Some((inode, mode, offset)) = current_file_descriptor_manager.file_descriptors.get(&fd) {
        match mode {
            OpenMode::Write | OpenMode::ReadWrite => Ok(inode.read().write_at(*offset, buf)),

            _ => Err(RcError::ACCESS_DENIED),
        }
    } else {
        Err(RcError::INVALID_ARGS)
    }
}

pub fn lseek(fd: FileDescriptor, offset: usize) -> RcResult<usize> {
    let current_process = get_current_process();
    let mut current_process = current_process.write();
    let current_file_descriptor_manager = &mut current_process.file_descriptor_manager;

    let (_, _, old_offset) = current_file_descriptor_manager
        .file_descriptors
        .get_mut(&fd)
        .ok_or(RcError::INVALID_ARGS)?;
    *old_offset = offset;

    Ok(offset)
}

pub fn close(fd: FileDescriptor) -> RcResult<usize> {
    let current_process = get_current_process();
    let mut current_process = current_process.write();
    let current_file_descriptor_manager = &mut current_process.file_descriptor_manager;

    current_file_descriptor_manager
        .file_descriptors
        .remove(&fd)
        .ok_or(RcError::INVALID_ARGS)?;

    Ok(fd)
}

pub fn fsize(fd: FileDescriptor) -> RcResult<usize> {
    let current_process = get_current_process();
    let mut current_process = current_process.write();
    let current_file_descriptor_manager = &mut current_process.file_descriptor_manager;

    let (inode, _, _) = current_file_descriptor_manager
        .file_descriptors
        .get_mut(&fd)
        .ok_or(RcError::INVALID_ARGS)?;

    let size = inode.read().len();

    Ok(size)
}
