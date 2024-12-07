use core::iter::FusedIterator;

use alloc::{collections::btree_map::BTreeMap, string::String, vec::Vec};
use spin::Mutex;

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct ArgInfo {
    pub argc: usize,
    pub argv: usize,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct EnvInfo {
    pub env_start: usize,
    pub env_len: usize,
}

pub fn var(name: String) -> Option<String> {
    let EnvInfo { env_start, env_len } = *ENV_INFO.lock();

    let mut index = 0;
    let mut string = Vec::new();
    while index < env_len {
        let byte = unsafe { (env_start as *const u8).add(index).read() };
        index += 1;
        if byte == 0 {
            let mut string = String::from_utf8(string.clone()).unwrap();
            if string.starts_with(&(name.clone() + "=")) {
                return Some(string.split_off(name.len() + 1));
            }
        }
        string.push(byte);
    }

    None
}

pub fn set_env(env: BTreeMap<String, String>) {
    let mut env_data = Vec::new();
    for (key, value) in env {
        env_data.append(&mut key.into_bytes());
        env_data.push(b'=');
        env_data.append(&mut value.into_bytes());
        env_data.push(0);
    }

    let env_data = env_data.leak();
    let env_start = env_data.as_ptr() as usize;
    let env_len = env_data.len();

    const SET_ENV_SYSCALL: u64 = 20;

    syscall_macro::syscall!(SET_ENV_SYSCALL, env_start, env_len, 0, 0, 0).unwrap();

    *ENV_INFO.lock() = EnvInfo { env_start, env_len };
}

pub(crate) static ARG_INFO: Mutex<ArgInfo> = Mutex::new(ArgInfo { argc: 0, argv: 0 });
pub(crate) static ENV_INFO: Mutex<EnvInfo> = Mutex::new(EnvInfo {
    env_start: 0,
    env_len: 0,
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
