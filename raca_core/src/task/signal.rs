use alloc::{vec, vec::Vec};
use bit_field::BitField;

/// the signal structure.
#[derive(Debug, Clone, Copy)]
pub struct Signal {
    pub ty: usize,
    pub data: [u64; 8],
}

/// The signal manager.
/// You don't need to create one by yourself.
pub struct SignalManager {
    signal_bitmap: Bitmap,
    signals: Vec<Signal>,
    waiting_for: usize,
}

impl SignalManager {
    pub fn new(signal_type_num: usize) -> Self {
        Self {
            signal_bitmap: Bitmap::new(vec![0; signal_type_num]),
            signals: Vec::new(),
            waiting_for: 0,
        }
    }

    /// Returns whether the signal type is registered.
    pub fn has_signal(&self, signal_type: usize) -> bool {
        self.signal_bitmap.get(signal_type)
    }

    /// Registers a new signal and wakes up the process if it is waiting for the signal.
    pub fn register_signal(&mut self, signal_type: usize, signal: Signal) -> bool {
        assert_ne!(signal_type, 0);
        self.signal_bitmap.set(signal_type, true);
        self.signals.push(signal);

        if signal_type == self.waiting_for {
            self.waiting_for = 0;
            return true;
        }

        return false;
    }

    /// Starts to wait for a signal.
    pub fn register_wait_for(&mut self, signal_type: usize) {
        self.waiting_for = signal_type;
    }

    /// Gets a signal of the specified type. Returns None if the signal type is not registered.
    pub fn get_signal(&mut self, signal_type: usize) -> Option<Signal> {
        if self.signal_bitmap.get(signal_type) {
            for idx in 0..self.signals.len() {
                if self.signals[idx].ty == signal_type {
                    let signal = self.signals[idx];
                    return Some(signal);
                }
            }
            return None;
        } else {
            None
        }
    }

    /// Deletes a signal of the specified type. Nothing happens if the signal type is not registered.
    pub fn delete_signal(&mut self, signal_type: usize) {
        if self.signal_bitmap.get(signal_type) {
            self.signal_bitmap.set(signal_type, false);
            for idx in 0..self.signals.len() {
                if self.signals[idx].ty == signal_type {
                    self.signals.remove(idx);
                }
            }
        }
    }
}

pub struct Bitmap {
    inner: Vec<u8>,
}

impl Bitmap {
    pub fn new(mut inner: Vec<u8>) -> Self {
        inner.fill(0);
        Self { inner }
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn get(&self, index: usize) -> bool {
        let byte = self.inner[index / 8];
        byte.get_bit(index % 8)
    }

    pub fn set(&mut self, index: usize, value: bool) {
        let byte = &mut self.inner[index / 8];
        byte.set_bit(index % 8, value);
    }
}
