#![no_std]
#![no_main]
#![allow(unused_imports)]

use alloc::{string::String, vec};
use cara::{
    backend::{set_printer, Interpreter},
    frontend::{Lexer, Parser},
};
use raca_std::{
    exit,
    fs::{File, OpenMode},
    path::Path,
    print, println,
};

extern crate alloc;

#[no_mangle]
pub fn main() -> usize {
    //let mut path = String::new();
    //stdin().read_line(&mut path);
    let path = raca_std::env::args().nth(1).unwrap();

    let file = File::open(Path::new(path.clone()), OpenMode::Read);

    if let Err(_) = file {
        println!("File {} not found!", path);
        exit(1);
    }
    let file = file.unwrap();

    let mut data = vec![0; file.size().unwrap() as usize];
    file.read(&mut data);
    let code = String::from_utf8(data).unwrap();

    println!("good");

    //println!("code: {}", code);

    /*let code = r"
    fn fib(n) {
        if n == 1 {
            return 1;
        }
        if n == 2 {
            return 1;
        }
        var a = 1;
        var b = 1;
        for i in (3,n+1) {
            var t = a+b;
            a = b;
            b = t;
        }
        return b;
    }

    const test = fib(50000);
    print(test);

    ";*/

    let lexer = Lexer::new(code.into());
    let mut parser = Parser::new(lexer);

    let (ast, strings) = parser.parse_compile_unit();

    set_printer(|args| print!("{}", args));

    let mut interpreter = Interpreter::new(strings);

    match interpreter.visit(&ast) {
        #[cfg(debug_assertions)]
        Ok(value) => println!("{:?}", value),
        #[cfg(not(debug_assertions))]
        Ok(_) => (),
        Err(e) => {
            println!("on runtime error: {e}");
            println!("variables:");
            for (i, name) in interpreter.string_table().iter().enumerate() {
                println!(" {i}:\t{name}");
            }
            println!("{e}");
        }
    }

    0
}
