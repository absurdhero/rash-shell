
extern crate pretty_env_logger;

#[macro_use] extern crate log;
#[macro_use] extern crate nom;

use std::io;
use std::io::Read;
use std::io::Write;

pub mod parser;

fn main() {
    pretty_env_logger::init();


    let mut input = String::new();
    loop {
        print!("$ ");
        std::io::stdout().flush().expect("could not flush");

        let mut buffer: [u8; 1024] = [0; 1024];
        let len;
        match io::stdin().read(&mut buffer) {
            Ok(n) => {
                len = n;
                debug!("{} bytes read", len);
            }
            Err(_error) => std::process::exit(1),
        }


        let result = parser::parse(&buffer[..len]);

        match result {
            Ok((_in, command)) => eval(&command),
            Err(line) => println!("error: line {}", line),
        }

        input.clear();
    }
}

fn eval(command: &parser::Command) -> () {
    debug!("command: \"{:?}\"", command);
}