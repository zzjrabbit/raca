use alloc::vec;
use raca_std::{
    fs::{File, OpenMode},
    path::Path,
    task::{wait, Process},
};

use crate::CmdState;

pub fn try_run(path: Path) -> Option<CmdState> {
    if let Ok(mut file) = File::open(path, OpenMode::Read) {
        let mut buf = vec![0; file.size().unwrap() as usize];
        file.read(&mut buf).unwrap();
        file.close();

        let process = Process::new(&buf, "temp", 0, 0);
        process.run().unwrap();

        let code = wait().unwrap();
        Some(if code == 0 {
            CmdState::Ok
        } else {
            CmdState::Error
        })
    } else {
        None
    }
}
