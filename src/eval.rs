use ast;
use exec;
use nix::sys::wait;
use nix::unistd::*;
use std::env;
use std::ffi::CString;
use std::os::unix::io::RawFd;

pub fn eval(program: &ast::Program) -> i32 {
    let mut result = -1;
    for cc in &program.commands.complete_commands {
        result = complete_command(cc);
    }
    result
}

fn complete_command(cc: &ast::CompleteCommand) -> i32 {
    let mut result = -1;
    for (op, list) in &cc.and_ors {
        result = andor_list(*op == ast::TermOp::Amp, list);
    }
    result
}

fn andor_list(async: bool, list: &ast::AndOr) -> i32 {
    for (op, pipeline) in &list.pipelines {
        let result = exec_pipeline(async, pipeline);
        match op {
            ast::AndOrOp::And => {
                if result != 0 {
                    return result;
                }
            }
            ast::AndOrOp::Or => {
                if result == 0 {
                    return result;
                }
            }
        }
    }
    return -1;
}

fn exec_pipeline(async: bool, pipeline: &ast::Pipeline) -> i32 {
    let mut cmd_list: Vec<Pid> = vec![];
    let mut next_stdin: RawFd = 0;
    let mut cur_stdout: RawFd = 0;

    for i in 0..pipeline.commands.len() {
        let command = &pipeline.commands[i];
        debug!("{:?}", command);
        match command {
            ast::Command::Simple { assign, cmd, args } => {
                let parsed_cmd = match cmd {
                    ast::Arg::Arg(s) => eval_arg(*s),
                    // TODO: Evaluate the backquoted args as an andor_list and substitute stdout
                    ast::Arg::Backquote(_quoted_args) => CString::new("").unwrap(),
                };
                let mut parsed_args: Vec<CString> = vec![parsed_cmd.clone()];
                parsed_args.extend(args.iter().map(|a| {
                    match a {
                        ast::Arg::Arg(s) => eval_arg(*s),
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


                match fork() {
                    Ok(ForkResult::Parent { child }) => {
                        cmd_list.push(child);
                        if cur_stdin != 0 { close(cur_stdin).unwrap(); }
                        if cur_stdout != 1 { close(cur_stdout).unwrap(); }
                    }
                    Ok(ForkResult::Child) => {
                        dup2(cur_stdin, 0).expect("could not dup stdin");
                        dup2(cur_stdout, 1).expect("could not dup stdout");
                        // wire up stdin from last thing in pipeline and exec
                        if let Err(e) = exec::exec(&parsed_cmd, &parsed_args, &parsed_env) {
                            println!("could not exec: {}", e);
                            close(cur_stdin).unwrap();
                            close(cur_stdout).unwrap();
                        }
                    }
                    Err(_) => println!("Fork failed"),
                }
            }
        }
    }

    if async {
        return 0;
    }

    let mut return_status = -1;

    for child in cmd_list {
        let result = wait::waitpid(Some(child), None);
        match result {
            Ok(wait_status) => {
                match wait_status {
                    wait::WaitStatus::Exited(_pid, r) => return_status = r,
                    _ => (),
                }
            }
            Err(e) => {
                println!("wait failed: {}", e);
            }
        }
    }

    return return_status;
}

fn eval_arg(arg: &str) -> CString {
    if arg.as_bytes()[0] == b'$' {
        let key: &str = &arg[1..];
        match env::var(key) {
            Ok(v) => CString::new(v),
            Err(_) => CString::new(""),
        }
    } else {
        CString::new(arg)
    }.unwrap()
}