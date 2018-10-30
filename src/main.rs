
extern crate pretty_env_logger;
extern crate nix;
extern crate rustyline;
#[allow(unused)]
#[macro_use] extern crate log;
#[macro_use] extern crate lalrpop_util;

use rustyline::error::ReadlineError;

pub mod ast;
pub mod eval;

lalrpop_mod!(pub grammar);

fn main() {
    pretty_env_logger::init();

    let parser = grammar::programParser::new();

    let mut input: String = String::with_capacity(1024);

    let mut rl = rustyline::Editor::<()>::new();

    loop {

        let readline = rl.readline("$ ");
        match readline {
            Ok(line) => {
                input.push_str(line.as_str())
            },
            Err(ReadlineError::Interrupted) => {
                std::process::exit(1)
            },
            Err(ReadlineError::Eof) => {
                std::process::exit(1)
            },
            Err(err) => {
                println!("rash: error: {:?}", err);
                std::process::exit(1)
            }
        }

        match parser.parse(&input) {
            Ok(mut program) => {
                rl.add_history_entry(input.as_ref());
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
    }
}
