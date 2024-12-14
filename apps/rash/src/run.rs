use alloc::{
    string::String,
    vec::Vec,
};
use raca_std::{
    path::Path,
    process::{wait, Command},
};

use crate::CmdState;

pub fn try_run(args: Vec<String>) -> Option<CmdState> {
    let path = Path::new(args[0].clone());

    let mut cmd = Command::new(path.clone());
    for arg in args.iter().skip(1) {
        cmd.arg(arg.clone());
    }
    cmd.spawn().ok()?;
    wait().ok()?;
    Some(CmdState::Ok)
}
