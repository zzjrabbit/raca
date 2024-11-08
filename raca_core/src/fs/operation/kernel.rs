use crate::fs::{FileRef, Path};

use super::get_file_by_path;

pub fn kernel_open(path: Path) -> Option<FileRef> {
    get_file_by_path(path)
}
