use crate::{Errno, Result, mem::Vmar};
use alloc::{ffi::CString, vec::Vec};
use kernel_hal::mem::VirtAddr;
use pod::Pod;

impl Vmar {
    pub fn read_array<T: Pod>(&self, address: usize, len: usize) -> Result<Vec<T>> {
        let mut buffer = Vec::with_capacity(len);
        for id in 0..len {
            let value = self.read_val::<T>(address + id * size_of::<T>())?;
            buffer.push(value);
        }
        Ok(buffer)
    }

    pub fn read_array_map<T: Pod, R>(
        &self,
        address: usize,
        len: usize,
        f: impl Fn(T) -> Result<R>,
    ) -> Result<Vec<R>> {
        let mut buffer = Vec::with_capacity(len);
        for id in 0..len {
            let value = self.read_val::<T>(address + id * size_of::<T>())?;
            buffer.push(f(value)?);
        }
        Ok(buffer)
    }

    pub fn write_array<T: Pod>(&self, address: usize, data: &[T]) -> Result<()> {
        for (id, value) in data.iter().enumerate() {
            self.write_val(address + id * size_of::<T>(), value)?;
        }
        Ok(())
    }

    pub fn write_array_map<T, R: Pod>(
        &self,
        address: usize,
        data: &[T],
        f: impl Fn(&T) -> Result<R>,
    ) -> Result<()> {
        for (id, value) in data.iter().enumerate() {
            self.write_val(address + id * size_of::<T>(), &f(value)?)?;
        }
        Ok(())
    }

    pub fn read_val<T: Pod>(&self, address: usize) -> Result<T> {
        let mut buffer = T::new_zeroed();
        self.read(address, buffer.as_mut_bytes())?;
        Ok(buffer)
    }

    pub fn write_val<T: Pod>(&self, address: usize, value: &T) -> Result<()> {
        self.write(address, value.as_bytes())?;
        Ok(())
    }

    pub fn read(&self, address: usize, buffer: &mut [u8]) -> Result<()> {
        if let Some(child) = self.find_child(address) {
            return child.read(address, buffer);
        }

        let mut read: usize = 0;

        while read < buffer.len() {
            let current_address = address + read;

            let (mapping_start, mapping_size, vmo) = self
                .inner
                .read()
                .vm_mappings
                .iter()
                .find(|mapping| mapping.contains(current_address))
                .map(|mapping| (mapping.start(), mapping.size(), mapping.vmo().clone()))
                .unwrap();

            let remaining = buffer.len() - read;
            let chunk_size = mapping_size.min(remaining);

            vmo.read_bytes(
                current_address - mapping_start,
                &mut buffer[read..read + chunk_size],
            )?;
            read += chunk_size;
        }

        Ok(())
    }

    pub fn write(&self, address: usize, buffer: &[u8]) -> Result<()> {
        if let Some(child) = self.find_child(address) {
            return child.write(address, buffer);
        }

        let mut written: usize = 0;

        while written < buffer.len() {
            let current_address = address + written;

            let (mapping_start, mapping_size, vmo) = self
                .inner
                .read()
                .vm_mappings
                .iter()
                .find(|mapping| mapping.contains(current_address))
                .map(|mapping| (mapping.start(), mapping.size(), mapping.vmo().clone()))
                .ok_or(Errno::PageFault.no_message())?;

            let remaining = buffer.len() - written;
            let chunk_size = mapping_size.min(remaining);

            vmo.write_bytes(
                current_address - mapping_start,
                &buffer[written..written + chunk_size],
            )?;
            written += chunk_size;
        }

        Ok(())
    }
}

impl Vmar {
    pub fn read_cstring(
        &self,
        address: VirtAddr,
        max_string_len: Option<usize>,
    ) -> Result<CString> {
        let mut buffer = Vec::new();
        let mut current_address = address;

        loop {
            if current_address - address == max_string_len.unwrap_or(usize::MAX) {
                return Err(Errno::TooBig.no_message());
            }

            let byte: u8 = self.read_val(current_address)?;
            if byte == 0 {
                return CString::new(buffer).map_err(|_| Errno::InvArg.no_message());
            }
            buffer.push(byte);
            current_address += 1;
        }
    }
}
