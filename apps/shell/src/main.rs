#![no_std]
#![no_main]

use alloc::{
    format, string::{String, ToString}, vec
};
use core::fmt::Write;
use raca_std::{
    fs::{change_cwd, get_cwd, FileDescriptor, FileInfo, FileType, OpenMode},
    task::Process,
};

extern crate alloc;

fn shell_read_line(fd: &mut FileDescriptor, buf: &mut String) {
    buf.clear(); // make sure that the buf is clean

    let mut tmp_buf = [0; 1];
    fd.read(&mut tmp_buf);

    while tmp_buf[0] != b'\n' {
        if tmp_buf[0] == 8 {
            // backspace
            if let Some(_) = buf.pop() {
                write!(fd, "{} {}", 8 as char, 8 as char).unwrap();
            }
        } else {
            write!(fd, "{}", tmp_buf[0] as char).unwrap();
            buf.push(tmp_buf[0] as char);
        }
        fd.read(&mut tmp_buf);
    }
}

fn get_prompt() -> String {
    format!("\x1b[36m[\x1b[34mroot@raca \x1b[33m{}\x1b[36m]\x1b[34m:) \x1b[0m",get_cwd())
}

pub fn cat(stdin: FileDescriptor, file_path: String) {
    if let Ok(fd) = FileDescriptor::open(file_path.as_str(), OpenMode::Read) {
        let size = fd.size();
        let mut buf = vec![0; size];
        fd.read(buf.as_mut_slice());
        stdin.write(&buf);
        stdin.write(&[b'\n']);
    } else {
        stdin.write("Can't find the file.\n".as_bytes());
    }
}

pub fn write(stdin: FileDescriptor, file_path: String, text: String) {
    if let Ok(fd) = FileDescriptor::open(file_path.as_str(), OpenMode::Write) {
        fd.write(text.as_bytes());
    } else {
        stdin.write("Can't find the file.\n".as_bytes());
    }
}

pub fn ls(mut stdin: FileDescriptor,folder: String) {
    let infos = FileInfo::list(folder);
    for info in infos.iter() {
        match info.ty {
            FileType::Dir => write!(stdin, "\x1b[42m{}\x1b[0m ",info.name).unwrap(),
            FileType::File => write!(stdin, "\x1b[32m{}\x1b[0m ",info.name).unwrap(),
        }
    }
    writeln!(stdin).unwrap();
}

#[no_mangle]
pub fn main() {
    let mut fd = FileDescriptor::open("/dev/terminal", raca_std::fs::OpenMode::Write).unwrap();
    writeln!(fd, "\n\x1b[34mRACA-Shell \x1b[31mv0.1.0").unwrap();
    writeln!(
        fd,
        "\n\x1b[33mRemember to keep happy all the day when you open this shell! :)\n"
    )
    .unwrap();

    let mut input_buf = String::new();

    write!(fd, "{}", get_prompt()).unwrap();

    loop {
        shell_read_line(&mut fd, &mut input_buf);
        writeln!(fd).unwrap();

        let input =
            String::from_utf8(escape_bytes::unescape(input_buf.as_bytes()).unwrap()).unwrap();

        if input == "Avada Kedavra!" {
            writeln!(
                fd,
                "Oh! Don't try to kill anyone! We must be a good guy you know."
            )
            .unwrap();
        } else if input.starts_with("cat ") {
            if let Some(path) = input.split(" ").nth(1) {
                cat(fd, path.to_string());
            } else {
                writeln!(fd, "Expected a argument.").unwrap();
            }
        } else if input.starts_with("write ") {
            let mut input = input.split(" ");

            input.next();

            if let Some(path) = input.next() {
                let mut text = String::new();

                for i in input {
                    text += i;
                    text += " ";
                }

                write(fd, path.to_string(), text);
            } else {
                writeln!(fd, "Expected a argument.").unwrap();
            }
        } else if input.starts_with("echo ") {
            let mut string = input.clone();
            for _ in 0..5 {
                string.remove(0);
            }
            writeln!(fd, "{}", string).unwrap();
        } else if input.starts_with("run ") {
            if let Some(path) = input.split(" ").nth(1) {
                if let Ok(mut file) = FileDescriptor::open(path, OpenMode::Read) {
                    let mut buf = vec![0; file.size()];
                    file.read(&mut buf);
                    file.close();
                    //let (pipe1_read,pipe1_write) = FileDescriptor::open_pipe().unwrap();
                    //let (pipe2_read,pipe2_write) = FileDescriptor::open_pipe().unwrap();

                    let process = Process::new(&buf, "temp", 0, 0);
                    process.run();
                    //loop {
                    //    let mut buf = [0;1];
                    //    pipe2_read.read(&mut buf);
                    //    write!(fd, "{}", buf[0] as char).unwrap();
                    //}
                    loop {}
                }
            } else {
                writeln!(fd, "Expected a argument.").unwrap();
            }
        } else if input.starts_with("ls") {
            ls(fd, get_cwd());
        } else if input.starts_with("cd ") {
            if let Some(path) = input.split(" ").nth(1) {
                change_cwd(String::from(path));
            } else {
                writeln!(fd, "Expected a argument.").unwrap();
            }
        } else {
            writeln!(fd, "\x1b[31mBad Command: \x1b[0m{}\x1b[0m", input).unwrap();
        }
        write!(fd, "\x1b[0m{}", get_prompt()).unwrap();
    }
}
