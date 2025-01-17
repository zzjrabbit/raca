#![no_std]
#![no_main]

use alloc::{
    collections::btree_map::BTreeMap,
    format,
    string::{String, ToString},
    vec::Vec,
};
use raca_std::{path::Path, print, println};

extern crate alloc;

mod commands;
mod readline;
mod run;

fn get_cwd() -> Path {
    Path::new("/")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CmdState {
    Ok,
    Error,
}

fn get_prompt(state: CmdState) -> String {
    if state == CmdState::Ok {
        format!(
            "\x1b[34m╭─ \x1b[33m{} \x1b[0mat \x1b[34mroot@raca \n╰─\x1b[32m> \x1b[0m",
            get_cwd()
        )
    } else {
        format!(
            "\x1b[34m╭─ \x1b[33m{} \x1b[0mat \x1b[34mroot@raca \n╰─\x1b[31m> \x1b[0m",
            get_cwd()
        )
    }
}

type CommandFunction = fn(args: Vec<String>);

fn test() {
    loop {}
}

#[no_mangle]
pub fn main() -> usize {
    println!(
        "\n\x1b[34mRACA-Shell \x1b[31mv{}",
        env!("CARGO_PKG_VERSION")
    );
    
    let mut command_function_list = BTreeMap::<&str, CommandFunction>::new();

    {
        use commands::*;
        command_function_list.insert("exit", exit);
    }

    let mut readline = readline::Readline::new();

    println!("\n\x1b[33mRemember to keep happy all the day when you open this shell! :)\n");

    println!(
        "args: {:?}\npath: {:?}",
        raca_std::env::args().collect::<Vec<_>>(),
        raca_std::env::var("PATH")
    );

    let mut state = CmdState::Ok;

    print!("{}", get_prompt(state));

    raca_std::thread::spawn(test).unwrap();

    loop {
        let mut input_buf = String::new();

        //stdin().read_line(&mut input_buf);
        readline.read_line(&mut input_buf);

        let input =
            String::from_utf8(escape_bytes::unescape(input_buf.as_bytes()).unwrap()).unwrap();

        let args = input.split(" ").map(|x| x.to_string()).collect::<Vec<_>>();

        let function = command_function_list.get(&args[0].as_str());

        let mut args_ = Vec::new();
        for idx in 1..args.len() {
            args_.push(args[idx].clone());
        }

        if let Some(function) = function {
            function(args_);
        } else if let Some(state_) = run::try_run(args.clone()) {
            state = state_;
        } else {
            if input_buf.len() > 0 {
                println!("rash: command not found: \x1b[31m{}\x1b[0m", args[0]);
                state = CmdState::Error;
            }
        }

        print!("\x1b[0m{}", get_prompt(state));

        state = CmdState::Ok;
    }
}
