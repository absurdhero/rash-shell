// Copyright 2018 The Rash Project Developers. See the AUTHORS
// file at the top of this distribution for a list of copyright
// holders.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0

#[derive(Debug, PartialEq)]
pub struct Program<'a> {
    pub commands: CompleteCommands<'a>,
}

#[derive(Debug, PartialEq)]
pub struct CompleteCommands<'a> {
    pub complete_commands: Vec<CompleteCommand<'a>>,
}

impl<'a> CompleteCommands<'a> {
    pub fn push(
        mut self: CompleteCommands<'a>,
        element: CompleteCommand<'a>,
    ) -> CompleteCommands<'a> {
        self.complete_commands.push(element);
        self
    }
}

#[derive(Debug, PartialEq)]
pub struct CompleteCommand<'a> {
    pub and_ors: Vec<(TermOp, AndOr<'a>)>,
}

impl<'a> CompleteCommand<'a> {
    pub fn push(mut self, op: TermOp, element: AndOr<'a>) -> CompleteCommand<'a> {
        // update the TermOp of the previous list entry
        self.update_last(op);
        // add the new entry and assume it ends with a semicolon
        self.and_ors.push((TermOp::Semi, element));
        self
    }

    pub fn update_last(&mut self, op: TermOp) {
        if let Some((_, e)) = self.and_ors.pop() {
            self.and_ors.push((op, e));
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct AndOr<'a> {
    pub pipelines: Vec<(AndOrOp, Pipeline<'a>)>,
}

impl<'a> AndOr<'a> {
    pub fn push(mut self, op: AndOrOp, element: Pipeline<'a>) -> AndOr<'a> {
        if let Some((_, e)) = self.pipelines.pop() {
            self.pipelines.push((op, e));
        }
        self.pipelines.push((AndOrOp::And, element));
        self
    }
}

#[derive(Debug, PartialEq)]
pub enum AndOrOp {
    And,
    Or,
}

#[derive(Debug, PartialEq)]
pub struct Pipeline<'a> {
    pub commands: Vec<Command<'a>>,
    pub negated: bool,
}

impl<'a> Pipeline<'a> {
    pub fn new(cmd: Command<'a>) -> Pipeline<'a> {
        Pipeline {
            commands: vec![cmd],
            negated: false,
        }
    }

    pub fn negate(mut self) -> Pipeline<'a> {
        self.negated = !self.negated;
        self
    }

    pub fn push(mut self, cmd: Command<'a>) -> Pipeline<'a> {
        self.commands.push(cmd);
        self
    }
}

#[derive(Debug, PartialEq)]
pub enum Command<'a> {
    Simple {
        assign: Vec<&'a str>,
        cmd: Arg<'a>,
        args: Vec<Arg<'a>>,
        //redirect: Vec<Redirect<'a>>,
    },
}

#[derive(Debug, PartialEq)]
pub enum Arg<'a> {
    Arg(&'a str),
}

#[derive(Debug, PartialEq)]
pub enum TermOp {
    Semi,
    Amp,
}

//pub struct Redirect<'a> {
//    command: &'a Command<'a>,
//    operator: RedirectionType,
//    fname: Arg<'a>,
//    fd: int,
//    dupfd: int,
//}

//pub enum RedirectionType {
//    TO,     // fd > fname
//    CLOBBER,// fd >| fname
//    FROM,   // fd < fname
//    FROMTO, // fd <> fname
//    APPEND, // fd >> fname
//    TOFD,   // fd <& dupfd
//    FROMFD, // fd >& dupfd
//}

#[cfg(test)]
#[allow(unused_imports)]
mod tests {
    use lalrpop_util::lexer::Token;
    use lalrpop_util::ParseError;

    use crate::lexer::{LexError, Tok};
    use crate::{ast, grammar, lexer};

    use super::*;

    fn try_parse(input: &str) -> Result<ast::Program, ParseError<usize, Tok, LexError>> {
        let parser = grammar::programParser::new();
        let lexer = lexer::Lexer::new(input);
        return parser.parse(input, lexer);
    }

    fn parse(input: &str) -> ast::Program<'_> {
        return try_parse(input).unwrap();
    }

    fn complete_command<'a>(program: &'a ast::Program) -> &'a Vec<(TermOp, AndOr<'a>)> {
        return &program.commands.complete_commands.get(0).unwrap().and_ors;
    }

    fn single_command<'a>(program: &'a ast::Program) -> &'a Command<'a> {
        return &program.commands.complete_commands[0].and_ors[0].1.pipelines[0]
            .1
            .commands[0];
    }

    #[test]
    fn valid_commands() {
        assert!(try_parse("test\n").is_ok());
        assert!(try_parse("test foo &\n").is_ok());
        assert!(try_parse("test | | \n").is_err());
    }

    #[test]
    fn semicolon_delimiter() {
        // these should parse as two commands
        assert_eq!(complete_command(&parse("echo foo; bar")).len(), 2);
        assert_eq!(complete_command(&parse("echo foo;bar")).len(), 2);
        assert_eq!(complete_command(&parse("echo $foo;bar")).len(), 2);

        // these should parse as one command
        assert_eq!(complete_command(&parse("echo \"foo; bar\"")).len(), 1);
        assert_eq!(complete_command(&parse("echo 'foo; bar'")).len(), 1);
        assert_eq!(complete_command(&parse("echo `foo; bar`")).len(), 1);
    }

    #[test]
    fn simple_argument_parsing() {
        // argument parsing
        let program = parse("echo");
        let Command::Simple {
            args,
            assign: _,
            cmd,
        } = single_command(&program);
        assert_eq!(cmd, &Arg::Arg("echo"));
        assert_eq!(args.len(), 0);

        let program = parse("echo foo");
        let Command::Simple {
            args,
            assign: _,
            cmd: _,
        } = single_command(&program);
        assert_eq!(args.len(), 1);

        let program = parse("echo \"foo\"");
        let Command::Simple {
            args,
            assign: _,
            cmd: _,
        } = single_command(&program);
        assert_eq!(args.len(), 1);

        // Fails because the lexer delimits tokens on "
        // let program = parse("echo \"foo\"bar");
        // let Command::Simple { args, assign: _, cmd: _ } = single_command(&program);
        // assert_eq!(args.len(), 1);

        let program = parse("echo \"foo\" bar");
        let Command::Simple {
            args,
            assign: _,
            cmd: _,
        } = single_command(&program);
        assert_eq!(args.len(), 2);
    }
}
