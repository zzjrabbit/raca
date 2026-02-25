use core::fmt::{Error, Write};

use alloc::fmt;

use crate::debug;

struct Writer;

impl fmt::Write for Writer {
    #[inline]
    fn write_str(&mut self, s: &str) -> fmt::Result {
        debug(s).map_err(|_| Error)?;
        Ok(())
    }
}

#[inline]
pub fn _print(args: fmt::Arguments) {
    Writer.write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => (
        $crate::_print(format_args!($($arg)*))
    )
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)))
}
