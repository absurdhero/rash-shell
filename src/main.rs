#[macro_use]
extern crate lalrpop_util;
#[macro_use]
extern crate log;
extern crate nix;
extern crate pretty_env_logger;
extern crate rustyline;

use rustyline::error::ReadlineError;

pub mod ast;
pub mod context;
pub mod eval;

lalrpop_mod!(pub grammar);


fn main() {
    pretty_env_logger::init();

    let mut context = context::Context {
        interactive: stdin_is_a_tty(),
        last_return: None,
    };

    let parser = grammar::programParser::new();

    let mut input: String = String::with_capacity(1024);

    let mut rl = rustyline::Editor::<()>::new();

    loop {
        let readline = rl.readline("$ ");
        match readline {
            Ok(line) => {
                input.push_str(line.as_str())
            }
            Err(ReadlineError::Interrupted) => {
                std::process::exit(1)
            }
            Err(ReadlineError::Eof) => {
                std::process::exit(1)
            }
            Err(err) => {
                println!("rash: error: {:?}", err);
                std::process::exit(1)
            }
        }

        if let Some(r) = run_command(&parser, &mut rl, &mut context, input.as_ref()) {
            context.last_return = Some(r);
        } else {
            continue;
        }

        input.clear();
    }
}

fn stdin_is_a_tty() -> bool {
    nix::unistd::isatty(0).unwrap()
}

fn run_command(parser: &grammar::programParser,
               rl: &mut rustyline::Editor<()>,
               context: &mut context::Context,
               input: &str) -> Option<i32> {
    match parser.parse(input) {
        Ok(mut program) => {
            if context.interactive {
                rl.add_history_entry(input);
            }
            return Some(eval::eval(&mut program));
        }
        Err(e) => {
            if let lalrpop_util::ParseError::UnrecognizedToken { token: None, expected: _ } = e {
                None
            } else {
                eprintln!("rash: {}", e);
                Some(-1)
            }
        }
    }
}
