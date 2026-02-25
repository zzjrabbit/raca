use core::{
    alloc::Layout,
    fmt::{self, Write},
    slice::{from_raw_parts, from_raw_parts_mut},
};

use alloc::{alloc::alloc, boxed::Box};
use os_terminal::{DrawTarget, Terminal, font::BitmapFont};
use spin::{Mutex, Once};
use ustd::vm::Vmo;

struct Display {
    width: usize,
    height: usize,
    buffer: &'static mut [u8],
    modifications: usize,
}

impl DrawTarget for Display {
    fn size(&self) -> (usize, usize) {
        (self.width, self.height)
    }

    fn draw_pixel(&mut self, x: usize, y: usize, rgb: os_terminal::Rgb) {
        let (r, g, b) = rgb;
        let color = u32::from_be_bytes([r, g, b, 0]);
        let offset = (y * self.width + x) * 4;
        self.buffer[offset..offset + 4].copy_from_slice(&color.to_be_bytes());
        self.modifications += 1;
    }
}

static TERMINAL: Once<Mutex<Terminal<Display>>> = Once::new();
static BUFFER: Once<&'static [u8]> = Once::new();
static FB: Once<Vmo> = Once::new();

pub fn init(vmo: Vmo, width: usize, height: usize) {
    let size = width * height * 4;

    let buffer_ptr = unsafe { alloc(Layout::from_size_align(size, 1).unwrap()) };
    let buffer = unsafe { from_raw_parts_mut(buffer_ptr, size) };

    BUFFER.call_once(|| unsafe { from_raw_parts(buffer_ptr, size) });
    let display = Display {
        width,
        height,
        buffer,
        modifications: 0,
    };
    let mut terminal = Terminal::new(display);
    terminal.set_font_manager(Box::new(BitmapFont));
    terminal.set_crnl_mapping(true);
    terminal.set_scroll_speed(5);
    terminal.set_auto_flush(false);

    terminal.set_pty_writer(Box::new(|s| {
        TERMINAL.get().unwrap().lock().write_str(s).unwrap()
    }));
    TERMINAL.call_once(|| Mutex::new(terminal));
    FB.call_once(|| vmo);
}

pub fn _print(args: fmt::Arguments) {
    let _ = TERMINAL.get().unwrap().lock().write_fmt(args);
    TERMINAL.get().unwrap().lock().flush();
    FB.get().unwrap().write(0, BUFFER.get().unwrap()).unwrap();
}

#[macro_export]
macro_rules! termp {
    ($($arg:tt)*) => (
        $crate::terminal::_print(format_args!($($arg)*))
    )
}

#[macro_export]
macro_rules! termpln {
    () => ($crate::termp!("\n"));
    ($($arg:tt)*) => ($crate::termp!("{}\n", format_args!($($arg)*)))
}
