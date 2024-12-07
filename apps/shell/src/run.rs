use alloc::{
    string::{String, ToString},
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
    if let Err(err) = cmd.spawn() {
        if err == "failed to open file" {
            let binary_paths = raca_std::env::var("PATH".into()).unwrap();
            for binary_path in binary_paths.split(':') {
                let absolute_path = Path::new(binary_path.to_string()).join(path.clone());
                let mut cmd = Command::new(absolute_path.clone());
                for arg in args.iter().skip(1) {
                    cmd.arg(arg.clone());
                }

                cmd.spawn().ok()?;
                wait().ok()?;
                return Some(CmdState::Ok);
            }
        }
    }
    wait().ok()?;
    Some(CmdState::Ok)
}
