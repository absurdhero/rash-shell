// Copyright 2018 The Rash Project Developers. See the AUTHORS
// file at the top of this distribution for a list of copyright
// holders.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0

#[macro_use]
extern crate lalrpop_util;
#[macro_use]
extern crate log;

use rustyline::error::ReadlineError;

pub mod ast;
pub mod builtins;
pub mod context;
pub mod environment;
pub mod eval;
pub mod exec;

lalrpop_mod!(pub grammar);


fn main() {
    pretty_env_logger::init();

    let context = context::Context {
        interactive: stdin_is_a_tty(),
        last_return: 0,
        builtins: builtins::Builtins::new(),
        env: environment::from_system(),
    };

    let mut eval = eval::Eval::new(context);

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

        if !run_command(&parser, &mut rl, &mut eval, input.as_ref()) {
            continue;
        }

        input.clear();
    }
}

fn stdin_is_a_tty() -> bool {
    nix::unistd::isatty(0).unwrap()
}

/// Parses and runs a command.
/// Returns false if the input is incomplete.
fn run_command(parser: &grammar::programParser,
               rl: &mut rustyline::Editor<()>,
               eval: &mut eval::Eval,
               input: &str) -> bool {
    match parser.parse(input) {
        Ok(mut program) => {
            if eval.context.interactive {
                rl.add_history_entry(input);
            }
            eval.eval(&mut program);
            true
        }
        Err(e) => {
            if let lalrpop_util::ParseError::UnrecognizedToken { token: None, expected: _ } = e {
                false
            } else {
                eprintln!("rash: {}", e);
                eval.context.last_return = 2;
                true
            }
        }
    }
}
