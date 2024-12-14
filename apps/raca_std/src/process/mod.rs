use alloc::{
    string::{String, ToString},
    vec,
    vec::Vec,
};

use crate::path::Path;

pub struct Command {
    path: Path,
    args: Vec<String>,
}

impl Command {
    pub fn new(path: Path) -> Self {
        Self {
            path: path.clone(),
            args: vec![path.into()],
        }
    }

    pub fn arg(&mut self, arg: String) -> &mut Self {
        self.args.push(arg);
        self
    }

    pub fn spawn(&self) -> Result<usize, String> {
        let mut args = Vec::new();
        for arg in self.args.iter() {
            args.push(arg.as_bytes().to_vec());
            args.push(vec![0]);
        }

        let mut cmd_line = Vec::new();
        for arg in self.args.iter() {
            cmd_line.append(&mut arg.as_bytes().to_vec());
            cmd_line.push(0);
        }

        let mut binary_file = crate::fs::File::open(self.path.clone(), crate::fs::OpenMode::Read)
            .or_else(|_| {
                let Some(binary_paths) = crate::env::var("PATH") else {
                    return Err("failed to open file".to_string());
                };
                for binary_path in binary_paths.split(':') {
                    let Ok(file) = crate::fs::File::open(
                        Path::new(binary_path.to_string()).join(self.path.clone()),
                        crate::fs::OpenMode::Read,
                    ) else {
                        continue;
                    };
                    return Ok(file);
                }
                return Err("failed to open file".to_string());
            })?;
        let mut binary_buf = vec![0; binary_file.size().unwrap() as usize];
        binary_file
            .read(&mut binary_buf)
            .map_err(|_| "failed to read file".to_string())?;
        binary_file.close();

        let env = crate::env::ENV_INFO.lock().get_bytes();
        let (env_addr, env_len) = (env.as_ptr() as usize, env.len());

        let process =
            crate::task::Process::new(&binary_buf, "temp", 0, 0, &cmd_line, env_addr, env_len);
        process.run().map_err(|_| "failed to run process".into())
    }
}

pub use crate::task::{done_signal, exit, get_signal, start_wait_for_signal, wait};
