use core::iter::FusedIterator;

use alloc::{string::String, vec::Vec};
use spin::Mutex;

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct ArgInfo {
    pub argc: usize,
    pub argv: usize,
}

#[derive(Clone, Debug)]
pub(crate) struct EnvInfo {
    pub key_value: Vec<(String, String)>,
}

impl EnvInfo {
    pub fn get_bytes(&self) -> Vec<u8> {
        let mut data = Vec::new();
        for (key, value) in &self.key_value {
            data.append(&mut key.clone().into_bytes());
            data.push(b'=');
            data.append(&mut value.clone().into_bytes());
            data.push(0);
        }
        data
    }
    
    pub fn copy_from_slice(&mut self,data: &[u8]) {
        let mut index = 0;
        while index < data.len() {
            let key = {
                let mut key_data = Vec::new();
                while data[index] != b'=' {
                    key_data.push(data[index]);
                    index += 1;
                }
                String::from_utf8(key_data).unwrap()
            };
            index += 1;
            let value = {
                let mut value_data = Vec::new();
                while data[index] != 0 {
                    value_data.push(data[index]);
                    index += 1;
                }
                String::from_utf8(value_data).unwrap()
            };
            index += value.len() + 1;
            self.key_value.push((key, value));
        }
    }
}

pub fn var<T: Into<String>>(name: T) -> Option<String> {
    let name = name.into();
    ENV_INFO.lock().key_value.iter().find(|x| x.0.eq(&name)).cloned().map(|x| x.1)
}

pub fn set_var<K: Into<String>, V: Into<String>>(name: K, value: V) {
    let name = name.into();
    if let Some(index) = ENV_INFO.lock().key_value.iter().position(|x| x.0.eq(&name)) {
        ENV_INFO.lock().key_value[index] = (name, value.into());
    } else {
        ENV_INFO.lock().key_value.push((name, value.into()));
    }
}

pub(crate) static ARG_INFO: Mutex<ArgInfo> = Mutex::new(ArgInfo { argc: 0, argv: 0 });
pub(crate) static ENV_INFO: Mutex<EnvInfo> = Mutex::new(EnvInfo {
    key_value: Vec::new(),
});

pub fn args() -> Args {
    let ArgInfo { argc, argv } = *ARG_INFO.lock();

    Args::new(argv, argc)
}

pub struct Args {
    argv: usize,
    argc: usize,
    index: usize,
}

impl Args {
    fn new(argv: usize, argc: usize) -> Self {
        Self {
            argv,
            argc,
            index: 0,
        }
    }

    fn read_next(&mut self) -> Option<u8> {
        let byte = unsafe { (self.argv as *const u8).read() };
        self.argv += 1;
        if byte == 0 { None } else { Some(byte) }
    }

    fn read_last(&mut self) -> Option<u8> {
        let byte = unsafe { (self.argv as *const u8).read() };
        self.argv -= 1;
        if byte == 0 { None } else { Some(byte) }
    }
}

impl Iterator for Args {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.argc {
            return None;
        }

        let mut arg = Vec::new();
        self.index += 1;

        while let Some(byte) = self.read_next() {
            arg.push(byte);
        }

        Some(String::from_utf8(arg).unwrap())
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.argc, Some(self.argc))
    }
}

impl ExactSizeIterator for Args {
    fn len(&self) -> usize {
        self.argc
    }
    fn is_empty(&self) -> bool {
        self.argc == 0
    }
}

impl DoubleEndedIterator for Args {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.index == 0 {
            return None;
        }

        self.index -= 1;

        let mut arg = Vec::new();
        self.argv -= 2;
        while let Some(byte) = self.read_last() {
            arg.push(byte);
        }
        Some(String::from_utf8(arg).unwrap())
    }
}

impl FusedIterator for Args {}
