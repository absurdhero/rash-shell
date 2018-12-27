// Copyright 2018 The Rash Project Developers. See the AUTHORS
// file at the top of this distribution for a list of copyright
// holders.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0

use std::ffi::CString;
use std::os::unix::io::RawFd;

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
            let command = &pipeline.commands[i];
            debug!("{:?}", command);
            match command {
                ast::Command::Simple { assign, cmd, args } => {
                    let parsed_cmd = match cmd {
                        ast::Arg::Arg(s) => self.eval_arg(*s),
                        // TODO: Evaluate the backquoted args as an andor_list and substitute stdout
                        ast::Arg::Backquote(_quoted_args) => CString::new("").unwrap(),
                    };
                    let mut parsed_args: Vec<CString> = vec![parsed_cmd.clone()];
                    parsed_args.extend(args.iter().map(|a| {
                        match a {
                            ast::Arg::Arg(s) => self.eval_arg(*s),
                            ast::Arg::Backquote(_quoted_args) => CString::new("").unwrap(),
                        }
                    }));
                    let parsed_env: Vec<CString> = assign.iter().map(|a| {
                        CString::new(*a).unwrap()
                    }).collect();

                    let cur_stdin: RawFd;

                    if i == 0 {
                        cur_stdin = 0;
                        next_stdin = 0;
                        if pipeline.commands.len() == 1 {
                            cur_stdout = 1;
                        } else {
                            if let Ok((r, w)) = pipe() {
                                cur_stdout = w;
                                next_stdin = r;
                            }
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

                    if let Some(pid) = exec::run_command(&mut self.context, &parsed_cmd, &parsed_args, &parsed_env,
                                                         context::StdIO { stdin: cur_stdin, stdout: cur_stdout, stderr: 2 }) {
                        child_list.push(pid);
                    } else {
                        // if the last element in a pipeline is a built-in, record the return value
                        final_return = Some(self.context.last_return);
                    }
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
                    match wait_status {
                        wait::WaitStatus::Exited(_pid, r) => { final_return.get_or_insert(r); },
                        _ => {},
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

    fn eval_arg(&self, arg: &str) -> CString {
        if arg.as_bytes()[0] == b'$' {
            let key: &str = &arg[1..];
            match key {
                "?" => CString::new(format!("{}", self.context.last_return)),
                _ =>
                    match self.context.env.get(key) {
                        Some(v) => CString::new(v),
                        None => CString::new(""),
                    }
            }
        } else {
            CString::new(arg)
        }.unwrap()
    }
}
