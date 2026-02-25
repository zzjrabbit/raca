use errors::Result;
use pod::Pod;

use crate::vm::{MMUFlags, Vmar, Vmo};

static USER_STACK_SIZE: usize = 16 * 1024 * 1024;

pub fn new_user_stack(vmar: &Vmar) -> Result<(Vmo, Vmar)> {
    let stack = vmar.allocate(USER_STACK_SIZE)?;
    let vmo = Vmo::allocate(stack.page_count())?;
    stack.map(0, &vmo, MMUFlags::DATA)?;

    Ok((vmo, stack))
}

pub fn push_stack<T: Pod>(
    vmar: &Vmar,
    vmo: &Vmo,
    stack_ptr: &mut usize,
    data: &T,
) -> Result<usize> {
    *stack_ptr -= size_of::<T>();
    let data_ptr = *stack_ptr;
    vmo.write_val(data_ptr - vmar.base(), data)?;
    Ok(data_ptr)
}
