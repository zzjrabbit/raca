use alloc::string::String;
use spin::Mutex;

use crate::fs::InodeFunction;

pub static KEYBOARD_INPUT: Mutex<String> = Mutex::new(String::new());

pub struct Terminal;

impl InodeFunction for Terminal {
    fn read_at(&self, _offset: usize, data: &mut [u8]) -> usize {
        let mut keyboard_input = KEYBOARD_INPUT.lock();

        let len = data.len().min(keyboard_input.len());

        for idx in 0..len {
            data[idx] = keyboard_input.remove(0) as u8;
        }

        len
    }

    fn write_at(&mut self, _offset: usize, data: &[u8]) -> usize {
        let string = String::from_utf8_lossy_owned(data.to_vec());
        crate::print!("{}", string);
        //log::info!("Done");
        data.len()
    }

    fn len(&self) -> usize {
        KEYBOARD_INPUT.lock().len()
    }
}
