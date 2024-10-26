#![allow(improper_ctypes)]

use core::fmt::{self, Write};

use spin::Mutex;
use x86_64::instructions::interrupts;

extern "C" {
    fn print(s: &str);
}

struct Writer;

impl Write for Writer {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        unsafe {
            // Unknown Bug
            // Don't delete these two lines
            print(" ");
            print((8 as char).encode_utf8(&mut [0; 4]));
            print(s);
        }
        Ok(())
    }
}

static TERMINAL: Mutex<Writer> = Mutex::new(Writer);

#[inline]
pub fn _print(args: fmt::Arguments) {
    interrupts::without_interrupts(|| {
        TERMINAL.lock().write_fmt(args).unwrap();
    });
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => (
        $crate::io::_print(
            format_args!($($arg)*)
        )
    )
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)))
}
