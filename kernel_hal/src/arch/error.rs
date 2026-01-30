use loongarch64::structures::paging::{MapToError, PageSize, TranslateError};

use crate::{Errno, Error};

impl<S: PageSize> From<MapToError<S>> for Error {
    fn from(value: MapToError<S>) -> Self {
        match value {
            MapToError::FrameAllocationFailed => {
                Errno::ENOMEM.with_message("Frame allocation failed.")
            }
            MapToError::PageAlreadyMapped(_) => {
                Errno::EALREADY.with_message("Page already mapped.")
            }
            MapToError::ParentEntryHugePage => {
                Errno::EINVAL.with_message("Parent entry is a huge page.")
            }
        }
    }
}

impl From<TranslateError> for Error {
    fn from(value: TranslateError) -> Self {
        match value {
            TranslateError::InvalidFrameAddress(_) => {
                Errno::EINVAL.with_message("Invalid frame address.")
            }
            TranslateError::PageNotMapped => Errno::EINVAL.with_message("Page not mapped."),
            _ => unreachable!(),
        }
    }
}
