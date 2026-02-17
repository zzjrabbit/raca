use acpi::aml::AmlError;
use acpi::{PciAddress, PhysicalMapping};
use core::ptr::NonNull;

use crate::arch::mem::flush_cache;
use crate::mem::{PhysAddr, VirtAddr, phys_to_virt};

#[derive(Clone, Copy)]
pub struct AcpiHandler;

impl AcpiHandler {
    fn read<T>(&self, address: usize) -> T {
        let address = address as PhysAddr;
        let address: *const T = phys_to_virt(address) as *const T;
        flush_cache(address as VirtAddr);
        unsafe { address.read_volatile() }
    }

    fn write<T>(&self, address: usize, value: T) {
        let address = address as PhysAddr;
        let address: *mut T = phys_to_virt(address) as *mut T;
        unsafe {
            address.write_volatile(value);
        }
        flush_cache(address as VirtAddr);
    }

    fn read_io<T>(&self, _port: u16) -> T {
        unimplemented!()
    }

    fn write_io<T>(&self, _port: u16, _value: T) {
        unimplemented!()
    }

    fn read_pci<T>(&self, _address: PciAddress, _offset: u16) -> T {
        unimplemented!()
    }

    fn write_pci<T>(&self, _address: PciAddress, _offset: u16, _value: T) {
        unimplemented!()
    }
}

macro_rules! aml_io {
    ([$($op:ident)?], $size:ty, ($($v:tt: $t:ty),+)) => {
        pastey::paste! {
            fn [<read_ $($op _)? $size>](&self, $($v: $t),+) -> $size {
                self.[<read $(_ $op)?>]::<$size>($($v),+)
            }
            fn [<write_ $($op _)? $size>](&self, $($v: $t),+, value: $size) {
                self.[<write $(_ $op)?>]::<$size>($($v),+, value)
            }
        }
    };
}

impl acpi::Handler for AcpiHandler {
    unsafe fn map_physical_region<T>(
        &self,
        physical_address: usize,
        size: usize,
    ) -> PhysicalMapping<Self, T> {
        let physical_address = physical_address as PhysAddr;
        let virtual_address = phys_to_virt(physical_address);

        let virtual_start = unsafe { NonNull::new_unchecked(virtual_address as *mut T) };

        PhysicalMapping {
            physical_start: physical_address,
            virtual_start,
            region_length: size,
            mapped_length: size,
            handler: *self,
        }
    }

    fn unmap_physical_region<T>(_region: &PhysicalMapping<Self, T>) {}

    aml_io!([], u8, (address: usize));
    aml_io!([], u16, (address: usize));
    aml_io!([], u32, (address: usize));
    aml_io!([], u64, (address: usize));
    aml_io!([io], u8, (port: u16));
    aml_io!([io], u16, (port: u16));
    aml_io!([io], u32, (port: u16));
    aml_io!([pci], u8, (address: PciAddress, offset: u16));
    aml_io!([pci], u16, (address: PciAddress, offset: u16));
    aml_io!([pci], u32, (address: PciAddress, offset: u16));

    fn nanos_since_boot(&self) -> u64 {
        todo!()
    }

    fn stall(&self, _microseconds: u64) {
        todo!()
    }

    fn sleep(&self, _milliseconds: u64) {
        todo!()
    }

    fn create_mutex(&self) -> acpi::Handle {
        acpi::Handle(0)
    }

    fn acquire(&self, _mutex: acpi::Handle, _timeout: u16) -> Result<(), AmlError> {
        todo!()
    }

    fn release(&self, _mutex: acpi::Handle) {
        todo!()
    }
}
