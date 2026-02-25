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

pub fn get_vmar_base(process: &Arc<Process>, handle: u32) -> SyscallResult {
    let vmar = process.find_object_with_rights::<Vmar>(HandleId::from_raw(handle), Rights::READ)?;
    let base = vmar.base();
    Ok(base)
}

pub fn get_vmar_size(process: &Arc<Process>, handle: u32) -> SyscallResult {
    let vmar = process.find_object_with_rights::<Vmar>(HandleId::from_raw(handle), Rights::READ)?;
    let size = vmar.size();
    Ok(size)
}

pub fn allocate_vmo(process: &Arc<Process>, count: usize, handle_addr: usize) -> SyscallResult {
    let vmo = Vmo::allocate_ram(count)?;
    let handle = Handle::new(vmo, Rights::VMO);
    let handle = process.add_handle(handle);

    process.root_vmar().write_val(handle_addr, &handle)?;

    Ok(0)
}

pub fn read_vmo(
    process: &Arc<Process>,
    handle: u32,
    offset: usize,
    data_ptr: usize,
    len: usize,
) -> SyscallResult {
    let vmo = process.find_object_with_rights::<Vmo>(HandleId::from_raw(handle), Rights::READ)?;

    let mut buffer = alloc::vec![0u8; len];
    vmo.read_bytes(offset, &mut buffer)?;
    process.root_vmar().write(data_ptr, &buffer)?;

    Ok(0)
}

pub fn write_vmo(
    process: &Arc<Process>,
    handle: u32,
    offset: usize,
    data_ptr: usize,
    len: usize,
) -> SyscallResult {
    let vmo = process.find_object_with_rights::<Vmo>(HandleId::from_raw(handle), Rights::WRITE)?;

    let mut buffer = alloc::vec![0u8; len];
    process.root_vmar().read(data_ptr, &mut buffer)?;
    vmo.write_bytes(offset, &buffer)?;

    Ok(0)
}
