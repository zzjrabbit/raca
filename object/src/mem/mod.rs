use kernel_hal::mem::PageSize;

pub use vmar::Vmar;
pub use vmo::Vmo;

mod vmar;
mod vmo;

pub const PAGE_SIZE: usize = PageSize::Size4K as usize;

pub const fn align_down_by_page_size(addr: usize) -> usize {
    addr / PAGE_SIZE * PAGE_SIZE
}

pub const fn align_up_by_page_size(addr: usize) -> usize {
    align_down_by_page_size(addr + PAGE_SIZE - 1)
}
