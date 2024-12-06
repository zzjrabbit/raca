use core::iter::FusedIterator;

use alloc::{string::String, vec, vec::Vec};

pub fn args() -> Args {
    let cmd_line_file = crate::fs::File::open(
        crate::path::Path::new("/proc/cmd_line"),
        crate::fs::OpenMode::Read,
    )
    .unwrap();

    let mut data = vec![0; cmd_line_file.size().unwrap()];
    cmd_line_file.read(&mut data).unwrap();

    Args::new(data)
}

pub struct Args {
    data: Vec<u8>,
    index: usize,
    len: usize,
}

impl Args {
    fn new(data: Vec<u8>) -> Self {
        let mut len = 0;
        for i in 0..data.len() {
            if data[i] == 0 {
                len += 1;
                break;
            }
        }
        Self {
            data,
            index: 0,
            len,
        }
    }
}

impl Iterator for Args {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        let mut arg = Vec::new();
        while self.index < self.data.len() && self.data[self.index] != 0 {
            arg.push(self.data[self.index]);
            self.index += 1;
        }
        self.index += 1;
        if arg.len() == 0 {
            return None;
        }
        Some(String::from_utf8(arg).unwrap())
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.len))
    }
}

impl ExactSizeIterator for Args {
    fn len(&self) -> usize {
        self.len
    }
    fn is_empty(&self) -> bool {
        self.len == 0
    }
}

impl DoubleEndedIterator for Args {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.index == 0 {
            return None;
        }

        let mut arg = Vec::new();
        self.index -= 2;
        while self.data[self.index] != 0 {
            arg.push(self.data[self.index]);
            if self.index == 0 {
                break;
            }
            self.index -= 1;
        }
        if self.index > 0 {
            self.index += 1;
        }
        Some(String::from_utf8(arg).unwrap())
    }
}

impl FusedIterator for Args {}
