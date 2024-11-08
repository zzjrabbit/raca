use crate::{
    error::{RcError, RcResult},
    fs::Path,
    task::SCHEDULER,
};

use super::{get_file_by_path, FileDescriptor, OpenMode};

pub fn open(path: Path, mode: OpenMode) -> RcResult<FileDescriptor> {
    let current_process = SCHEDULER
        .lock()
        .current_thread()
        .upgrade()
        .unwrap()
        .read()
        .process
        .clone();

    Ok(
        current_process
        .upgrade()
        .unwrap()
        .write()
        .file_descriptor_manager
        .add_file(get_file_by_path(path).ok_or(RcError::NOT_FOUND)?, mode)
    )
}
