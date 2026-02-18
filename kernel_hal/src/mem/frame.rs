use core::fmt::Display;

use alloc::fmt;
use bit_field::BitField;
use humansize::{BINARY, format_size};

use crate::mem::PhysAddr;

pub struct Bitmap(&'static mut [usize]);

#[allow(dead_code)]
impl Bitmap {
    const BITS: usize = usize::BITS as usize;

    pub fn new(inner: &'static mut [usize]) -> Self {
        inner.fill(0);
        Self(inner)
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.0.len() * Self::BITS
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    #[inline]
    pub fn get(&self, index: usize) -> bool {
        let byte = self.0[index / Self::BITS];
        byte.get_bit(index % Self::BITS)
    }

    #[inline]
    pub fn set(&mut self, index: usize, value: bool) {
        let byte = &mut self.0[index / Self::BITS];
        byte.set_bit(index % Self::BITS, value);
    }
}

impl Bitmap {
    pub fn set_range(&mut self, start: usize, end: usize, value: bool) {
        if start >= end || start >= self.len() {
            return;
        }

        let start_byte = start.div_ceil(Self::BITS);
        let end_byte = end / Self::BITS;

        (start..(start_byte * Self::BITS).min(end)).for_each(|i| self.set(i, value));

        if start_byte > end_byte {
            return;
        }

        if start_byte <= end_byte {
            let fill_value = if value { usize::MAX } else { 0 };
            self.0[start_byte..end_byte].fill(fill_value);
        }

        ((end_byte * Self::BITS).max(start)..end).for_each(|i| self.set(i, value));
    }

    pub fn find_range(&mut self, length: usize, value: bool) -> Option<usize> {
        let byte_match = if value { usize::MAX } else { 0 };
        let (mut count, mut start_index) = (0, 0);

        for (index, &byte) in self.0.iter().enumerate() {
            match byte {
                b if b == !byte_match => count = 0,
                b if b == byte_match => {
                    if length < Self::BITS {
                        return Some(index * Self::BITS);
                    }
                    if count == 0 {
                        start_index = index * Self::BITS;
                    }
                    count += Self::BITS;
                    if count >= length {
                        return Some(start_index);
                    }
                }
                _ => {
                    for bit_index in 0..Self::BITS {
                        if byte.get_bit(bit_index) == value {
                            if count == 0 {
                                start_index = index * Self::BITS + bit_index;
                            }
                            count += 1;
                            if count == length {
                                return Some(start_index);
                            }
                        } else {
                            count = 0;
                        }
                    }
                }
            }
        }

        None
    }
}

pub struct BitmapFrameAllocator {
    bitmap: Bitmap,
    origin_frames: usize,
    usable_frames: usize,
}

impl BitmapFrameAllocator {
    #[inline]
    fn used_bytes(&self) -> usize {
        (self.origin_frames - self.usable_frames) * 4096
    }

    #[inline]
    fn total_bytes(&self) -> usize {
        self.origin_frames * 4096
    }
}

impl Display for BitmapFrameAllocator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} used, {} total",
            format_size(self.used_bytes(), BINARY),
            format_size(self.total_bytes(), BINARY)
        )
    }
}

impl BitmapFrameAllocator {
    /// Note that after initialization, you need to deallocate all the usable frames manually.
    pub fn new(bitmap_buffer: &'static mut [usize]) -> Self {
        let origin_frames = core::mem::size_of_val(bitmap_buffer) * 8;
        let bitmap = Bitmap::new(bitmap_buffer);

        BitmapFrameAllocator {
            bitmap,
            origin_frames,
            usable_frames: 0,
        }
    }

    pub fn new_all_free(bitmap_buffer: &'static mut [usize]) -> Self {
        bitmap_buffer.fill(usize::MAX);

        let origin_frames = core::mem::size_of_val(bitmap_buffer) * 8;
        let bitmap = Bitmap::new(bitmap_buffer);

        BitmapFrameAllocator {
            bitmap,
            origin_frames,
            usable_frames: origin_frames,
        }
    }

    pub fn allocate_frames(&mut self, count: usize) -> Option<PhysAddr> {
        let Some(index) = self.bitmap.find_range(count, true) else {
            log::error!(
                "No more usable frames! Usable: {}, required: {}",
                self.usable_frames,
                count
            );
            return None;
        };

        self.bitmap.set_range(index, index + count, false);
        self.usable_frames -= count;

        let address = index * 4096;
        Some(address)
    }

    pub fn deallocate_frames(&mut self, address: PhysAddr, count: usize) {
        let index = address / 4096;
        self.bitmap.set_range(index, index + count, true);
        self.usable_frames += count;
    }
}
