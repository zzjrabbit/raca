use alloc::sync::Arc;
use kernel_hal::mem::{CachePolicy, MMUFlags, PageProperty, Privilege};
use object::{
    mem::{Vmar, Vmo},
    object::{Handle, Rights},
    task::{HandleId, Process},
};

use crate::SyscallResult;

pub fn allocate_vmar(
    process: &Arc<Process>,
    handle: u32,
    size: usize,
    child_handle_addr: usize,
) -> SyscallResult {
    let vmar =
        process.find_object_with_rights::<Vmar>(HandleId::from_raw(handle), Rights::MANAGE)?;

    let child = vmar.allocate_child(size)?;
    let handle = Handle::new(child.clone(), Rights::VMAR);
    let handle = process.add_handle(handle);

    log::debug!("new vmar: {:#x}", handle.as_raw());
    process.root_vmar().write_val(child_handle_addr, &handle)?;

    Ok(child.base())
}

pub fn allocate_vmar_at(
    process: &Arc<Process>,
    handle: u32,
    addr: usize,
    size: usize,
    child_handle_addr: usize,
) -> SyscallResult {
    let vmar =
        process.find_object_with_rights::<Vmar>(HandleId::from_raw(handle), Rights::MANAGE)?;

    let child = vmar.create_child(addr, size)?;
    let handle = Handle::new(child, Rights::VMAR);
    let handle = process.add_handle(handle);

    process.root_vmar().write_val(child_handle_addr, &handle)?;

    Ok(0)
}

pub fn map_vmar(
    process: &Arc<Process>,
    handle: u32,
    offset: usize,
    vmo_handle: u32,
    flags: u32,
) -> SyscallResult {
    let vmar =
        process.find_object_with_rights::<Vmar>(HandleId::from_raw(handle), Rights::MANAGE)?;
    let vmo =
        process.find_object_with_rights::<Vmo>(HandleId::from_raw(vmo_handle), Rights::MAP)?;

    vmar.map(
        offset,
        &vmo,
        PageProperty::new(
            MMUFlags::from_bits_truncate(flags),
            CachePolicy::CacheCoherent,
            Privilege::User,
        ),
        true,
    )?;
    Ok(0)
}

pub fn unmap_vmar(process: &Arc<Process>, handle: u32, addr: usize, size: usize) -> SyscallResult {
    let vmar =
        process.find_object_with_rights::<Vmar>(HandleId::from_raw(handle), Rights::MANAGE)?;

    vmar.unmap(addr, size)?;
    Ok(0)
}

pub fn protect_vmar(
    process: &Arc<Process>,
    handle: u32,
    addr: usize,
    size: usize,
    flags: u32,
) -> SyscallResult {
    let vmar =
        process.find_object_with_rights::<Vmar>(HandleId::from_raw(handle), Rights::MANAGE)?;

    vmar.protect(addr, size, MMUFlags::from_bits_truncate(flags))?;
    Ok(0)
}

pub fn allocate_vmo(process: &Arc<Process>, count: usize, handle_addr: usize) -> SyscallResult {
    let vmo = Vmo::allocate_ram(count)?;
    let handle = Handle::new(vmo, Rights::VMAR);
    let handle = process.add_handle(handle);

    process.root_vmar().write_val(handle_addr, &handle)?;

    Ok(0)
}
