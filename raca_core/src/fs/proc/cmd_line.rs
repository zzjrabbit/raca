use crate::fs::InodeFunction;
use crate::task::{Process, SCHEDULER};
use alloc::{boxed::Box, sync::Arc};
use spin::RwLock;

pub struct CmdLine;

fn get_current_process() -> Arc<RwLock<Box<Process>>> {
    SCHEDULER
        .lock()
        .current_thread()
        .upgrade()
        .unwrap()
        .read()
        .process
        .upgrade()
        .unwrap()
}

impl InodeFunction for CmdLine {
    fn read_at(&self, _offset: usize, data: &mut [u8]) -> usize {
        let current_process = get_current_process();
        let current_process = current_process.read();

        let cmd_line = &current_process.cmd_line;

        let len = cmd_line.len();

        if len > data.len() {
            data.copy_from_slice(&cmd_line[..data.len()]);
            data.len()
        } else {
            data.copy_from_slice(&cmd_line[..len]);
            len
        }
    }

    fn write_at(&mut self, _offset: usize, _data: &[u8]) -> usize {
        0
    }

    fn len(&self) -> usize {
        let current_process = get_current_process();
        let current_process = current_process.read();

        current_process.cmd_line.len()
    }
}
