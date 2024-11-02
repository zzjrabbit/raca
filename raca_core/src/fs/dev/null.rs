use crate::fs::InodeFunction;

pub struct NullDevice;

impl InodeFunction for NullDevice {
    fn read_at(&self, _offset: usize, _data: &mut [u8]) -> usize {
        0
    }

    fn write_at(&mut self, _offset: usize, _data: &[u8]) -> usize {
        0
    }
}
