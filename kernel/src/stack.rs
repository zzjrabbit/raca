use alloc::sync::Arc;
use errors::Result;
use kernel_hal::mem::PageProperty;
use object::mem::{Vmar, Vmo};
use pod::Pod;

static USER_STACK_SIZE: usize = 8 * 1024 * 1024;

pub fn new_user_stack(vmar: Arc<Vmar>) -> Result<Arc<Vmar>> {
    let stack = vmar.allocate_child(USER_STACK_SIZE)?;
    let vmo = Vmo::allocate_ram(stack.page_count())?;
    stack.map(0, &vmo, PageProperty::user_data(), false)?;

    Ok(stack)
}

pub fn push_stack<T: Pod>(vmar: Arc<Vmar>, stack_ptr: &mut usize, data: &T) -> usize {
    *stack_ptr -= size_of::<T>();
    let data_ptr = *stack_ptr;
    vmar.write_val(data_ptr, data).unwrap();
    data_ptr
}
