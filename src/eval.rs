use ast;
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

                let res = Command::new(parsed_cmd)
                    .args(parsed_args)
                    .spawn();

                match res {
                    Ok(mut child) => {
                        let error_text = format!("could not execute {}", parsed_cmd);
                        let status = child.wait().expect(&error_text);
                        match status.code() {
                            // TODO: actually pipeline, don't just run the first command
                            Some(code) => return code,
                            None => println!("Process terminated by signal")
                        }
                    }
                    Err(e) => println!("{}", e),
                }
            }
        }
    }
    return -1;
}