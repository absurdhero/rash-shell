use ast;
use process;
use std::collections::HashMap;
use std::iter::FromIterator;
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
    for command in &pipeline.commands {
        debug!("{:?}", command);
        let mut cmd_list: Vec<process::ChildCommand> = vec![];
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

                let res = exec_simple(assign, parsed_cmd, &parsed_args);

                match res {
                    Ok(mut child) => {
                        cmd_list.push(child);
                    }
                    Err(e) => {
                        println!("{}", e);
                        return -1;
                    },
                }
            }
        }

        for child in &mut cmd_list {
            match child.process {
                process::ProcessType::Process(ref mut process) => {
                    let error_text = format!("could not wait for {}", child.cmd);
                    let status = process.wait().expect(&error_text);
                    match status.code() {
                        Some(code) => child.returned = Some(code),
                        None => println!("Process terminated by signal")
                    }
                },
                process::ProcessType::Builtin => {
                    println!("built-ins not supported");
                },
            }
        }
    }
    return -1;
}

pub fn exec_simple(assign: &Vec<&str>, cmd: &str, args: &Vec<&str>) -> std::io::Result<process::ChildCommand> {
    let env: HashMap<String, String> = HashMap::from_iter(
        assign.iter().map(|s| {
            let split: Vec<&str> = s.split('=').collect();
            (String::from(split[0]), String::from(split[1]))
        }));

    process::exec(&env, cmd, args)
}