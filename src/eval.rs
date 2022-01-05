// Copyright 2018 The Rash Project Developers. See the AUTHORS
// file at the top of this distribution for a list of copyright
// holders.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0

use std::iter::Peekable;
use std::os::unix::io::RawFd;
use std::str::Chars;

use nix::sys::wait;
use nix::unistd::*;

use crate::ast;
use crate::context;
use crate::exec;

pub struct Eval {
    pub context: context::Context,
}

impl Eval {
    pub fn new(context: context::Context) -> Eval {
        Eval { context }
    }

    pub fn eval(&mut self, program: &ast::Program) {
        for cc in &program.commands.complete_commands {
            self.complete_command(cc);
        }
    }

    fn complete_command(&mut self, cc: &ast::CompleteCommand) {
        for (op, list) in &cc.and_ors {
            self.andor_list(*op == ast::TermOp::Amp, list);
        }
    }

    fn andor_list(&mut self, exec_async: bool, list: &ast::AndOr) {
        for (op, pipeline) in &list.pipelines {
            self.exec_pipeline(exec_async, pipeline);
            match op {
                ast::AndOrOp::And => {
                    if self.context.last_return != 0 {
                        return;
                    }
                }
                ast::AndOrOp::Or => {
                    if self.context.last_return == 0 {
                        return;
                    }
                }
            }
        }
    }

    fn exec_pipeline(&mut self, exec_async: bool, pipeline: &ast::Pipeline) {
        let mut child_list: Vec<Pid> = vec![];
        let mut next_stdin: RawFd = 0;
        let mut cur_stdout: RawFd = 0;

        let mut final_return: Option<i32> = None;

        for i in 0..pipeline.commands.len() {
            // Create pipes between pipeline elements,
            // set up stdin on the first element and stdout on the last element
            let cur_stdin: RawFd;
            if i == 0 {
                cur_stdin = 0;
                next_stdin = 0;
                if pipeline.commands.len() == 1 {
                    cur_stdout = 1;
                } else if let Ok((r, w)) = pipe() {
                    cur_stdout = w;
                    next_stdin = r;
                }
            } else if i == pipeline.commands.len() - 1 {
                cur_stdin = next_stdin;
                cur_stdout = 1;
            } else {
                cur_stdin = next_stdin;
                if let Ok((r, w)) = pipe() {
                    cur_stdout = w;
                    next_stdin = r;
                }
            };

            let command = &pipeline.commands[i];
            debug!("{:?}", command);
            match command {
                ast::Command::Simple(ast::SimpleCommand { assign, cmd, args }) => {
                    let parsed_cmd = match cmd {
                        ast::Arg::Arg(s) => self.expand_arg(*s),
                    };

                    // assignments with no command change the current environment
                    if parsed_cmd.is_empty() {
                        for vareq in assign {
                            self.context.env.set_vareq(vareq);
                        }
                        return;
                    }

                    let mut parsed_args: Vec<String> = vec![parsed_cmd.clone()];
                    parsed_args.extend(args.iter().map(|a| match a {
                        ast::Arg::Arg(s) => self.expand_arg(*s),
                    }));

                    if let Some(pid) = exec::run_command(
                        &mut self.context,
                        &parsed_cmd,
                        &parsed_args,
                        assign,
                        context::StdIo {
                            stdin: cur_stdin,
                            stdout: cur_stdout,
                            stderr: 2,
                        },
                    ) {
                        child_list.push(pid);
                    } else {
                        // if the last element in a pipeline is a built-in, record the return value
                        final_return = Some(self.context.last_return);
                    }
                }
                ast::Command::Compound => {
                    panic!("unimplemented");
                }
            }
        }

        if exec_async {
            // async commands always return 0
            self.context.last_return = 0;
            return;
        }

        // Check children starting with the last one.
        // Save the return code if it wasn't saved already.
        for &child in child_list.iter().rev() {
            let result = wait::waitpid(Some(child), None);
            match result {
                Ok(wait_status) => {
                    if let wait::WaitStatus::Exited(_pid, r) = wait_status {
                        final_return.get_or_insert(r);
                    };
                }
                Err(e) => {
                    eprintln!("rash: wait failed: {}", e);
                }
            }
        }

        if let Some(i) = final_return {
            self.context.last_return = i;
        }
    }

    fn expand_arg(&self, arg: &str) -> String {
        let mut chars = arg.chars().peekable();
        let mut expanded = String::new();
        let mut quoted: Option<char> = None;
        let mut escaped = false;

        while chars.peek().is_some() {
            let c = chars.next().unwrap();

            if quoted.is_none() && !escaped {
                if c == '"' || c == '\'' {
                    quoted = Some(c);
                    continue;
                } else if c == '\\' {
                    escaped = true;
                    continue;
                } else if c == '`' {
                    quoted = Some(c);
                    continue;
                } else if c == '$' {
                    self.expand_var(&mut chars, &mut expanded);
                    continue;
                }

                expanded.push(c);
            } else if escaped {
                // immediately end escaping
                escaped = false;
                // handle escaped newline by "removing" the newline
                if c == '\n' {
                    continue;
                }
                expanded.push(c);
            } else if quoted == Some('\'') {
                if c == '\'' {
                    quoted = None;
                } else {
                    expanded.push(c);
                }
            } else if quoted == Some('`') {
                if c == '`' {
                    quoted = None;
                } else {
                    expanded.push(c);
                }
            } else if quoted == Some('"') {
                if c == '"' {
                    quoted = None;
                } else if c == '\\' {
                    escaped = true;
                } else if c == '$' {
                    self.expand_var(&mut chars, &mut expanded);
                } else {
                    expanded.push(c);
                }
            }
        }
        expanded
    }

    /// Expand $name, and ${param}, and ${param-default} forms.
    /// This is a small subset of the spec found at:
    ///  POSIX 2.6.2 Parameter Expansion
    fn expand_var(&self, chars: &mut Peekable<Chars>, expanded: &mut String) {
        let mut param = String::new();
        let mut default_val = String::new();

        let delimited = if chars.peek() == Some(&'{') {
            chars.next();
            true
        } else {
            false
        };

        if chars.peek() == Some(&'?') {
            chars.next();
            let return_val = format!("{}", self.context.last_return);
            expanded.push_str(&return_val);
            return;
        }

        while chars.peek().is_some() {
            let c = chars.next().unwrap();
            if delimited {
                if c == '}' {
                    break;
                }
                if c == '-' {
                    while chars.peek().is_some() {
                        let c = chars.next().unwrap();
                        if c == '}' {
                            break;
                        }
                        default_val.push(c);
                    }
                    break;
                }
                param.push(c);
            } else if c.is_alphanumeric() || c == '_' {
                param.push(c);
            } else {
                break;
            }
        }
        expanded.extend(self.context.env.get(&param).or(Some(&default_val)));
    }
}
