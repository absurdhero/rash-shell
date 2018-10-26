use ast;
use process;
use std::collections::HashMap;
use std::iter::FromIterator;
#[cfg(unix)]
use std::os::unix::io::*;
use std::process::*;

pub fn eval(program: &ast::Program) -> () {
    for cc in &program.commands.complete_commands {
        complete_command(cc);
    }
}

fn complete_command(cc: &ast::CompleteCommand) {
    for (op, list) in &cc.and_ors {
        andor_list(*op == ast::TermOp::Amp, list);
    }
}

fn andor_list(async: bool, list: &ast::AndOr) {
    // TODO: Implement async by spawning a thread instead of using before_exec (unix only)
    for (op, pipeline) in &list.pipelines {
        let result = exec_pipeline(pipeline);
        match op {
            ast::AndOrOp::And => {
                if result != 0 {
                    break;
                }
            }
            ast::AndOrOp::Or => {
                if result == 0 {
                    break;
                }
            }
        }
    }
}

fn exec_pipeline(pipeline: &ast::Pipeline) -> i32 {
    let mut cmd_list: Vec<process::ChildCommand> = vec![];
    let mut last_stdout = 1;

    for command in &pipeline.commands {
        debug!("{:?}", command);
        match command {
            ast::Command::Simple { assign, cmd, args } => {
                let parsed_cmd = match cmd {
                    ast::Arg::Arg(s) => s,
                    // TODO: Evaluate the backquoted args as an andor_list and substitute stdout
                    ast::Arg::Backquote(_quoted_args) => "",
                };
                let parsed_args: Vec<&str> = args.iter().map(|a| {
                    match a {
                        ast::Arg::Arg(s) => s,
                        ast::Arg::Backquote(_quoted_args) => "",
                    }
                }).collect();

                let res = exec_simple(assign, parsed_cmd, &parsed_args, last_stdout);

                match res {
                    Ok(mut child) => {
                        cmd_list.push(child);
                        if let process::ProcessType::Process(p) = &child {
                            if let Some(stdin) = p.stdin {
                                stdin.as_raw()
                            }
                        }
                    }
                    Err(e) => {
                        println!("{}", e);
                        return -1;
                    },
                }
            }
        }
    }
    let mut running = vec![];

    // start each process
    for child in &mut cmd_list {
        match child.process {
            process::ProcessType::Process(ref mut process) => {
                match process.spawn() {
                    Ok(mut c) => running.push(c),
                    Err(e) => {
                        println!("{}", e);
                        return -1;
                    }
                }
            },
            process::ProcessType::Builtin => {
                println!("built-ins not supported");
            },
        }
    }

    // wait for last process to finish
    let mut last_cmd = running.last().unwrap();
    let error_text = format!("could not wait for {}", last_cmd.cmd);
    let return_status = -1;
    let status = process.wait().expect(&error_text);
    match status.code() {
        Some(code) => last_cmd.returned = Some(code),
        None => println!("Process terminated by signal")
    }
    return -1;
}

pub fn exec_simple(assign: &Vec<&str>, cmd: &str, args: &Vec<&str>, stdin: i32, stdout: i32) -> std::io::Result<process::ChildCommand> {
    let env: HashMap<String, String> = HashMap::from_iter(
        assign.iter().map(|s| {
            let split: Vec<&str> = s.split('=').collect();
            (String::from(split[0]), String::from(split[1]))
        }));

    process::exec(&env, cmd, args, stdin, stdout)
}