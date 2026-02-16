use alloc::sync::Arc;
use errors::Result;
use kernel_hal::mem::PageProperty;
use object::mem::{Vmar, Vmo};

static USER_STACK_SIZE: usize = 8 * 1024 * 1024;

pub fn new_user_stack(vmar: Arc<Vmar>) -> Result<Arc<Vmar>> {
    let stack = vmar.allocate_child(USER_STACK_SIZE)?;
    let vmo = Vmo::allocate_ram(stack.page_count())?;
    stack.map(0, &vmo, PageProperty::user_data(), false)?;

    Ok(stack)
}
