
extern crate pretty_env_logger;
#[allow(unused)]
#[macro_use] extern crate log;
#[macro_use] extern crate lalrpop_util;

use std::io;
use std::io::Write;

pub mod ast;
pub mod eval;

lalrpop_mod!(pub grammar);

fn main() {
    pretty_env_logger::init();

    let parser = grammar::programParser::new();

    print!("$ ");

    let mut input: String = String::with_capacity(1024);

    loop {
        std::io::stdout().flush().expect("could not flush");

        match io::stdin().read_line(&mut input) {
            Ok(n) => {
                if n == 0 {
                    break;
                }
            }
            Err(_error) => std::process::exit(1),
        }

        //debug!("read: {:X?}", input);

        match parser.parse(&input) {
            Ok(mut program) => {
                eval::eval(&mut program);
            },
            Err(e) => {
                if let lalrpop_util::ParseError::UnrecognizedToken { token: Option::None, expected: _ } = e {
                        continue;
                } else {
                    println!("rash: {}", e)
                }
            },
        }

        input.clear();
        print!("$ ");
    }
}
