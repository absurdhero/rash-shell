use std::collections::HashMap;
use std::process::*;


pub struct ChildCommand {
    pub cmd: String,
    pub process: ProcessType,
    pub returned: Option<i32>,
}

pub enum ProcessType {
    Process(Command),
    Builtin,
}

pub fn exec(env: &HashMap<String, String>, cmd: &str, args: &Vec<&str>, stdin: i32, stdout: i32) -> std::io::Result<ChildCommand> {
    let command = Command::new(cmd)
        .envs(env)
        .args(args)
        .stdin(Stdio::piped());
    return command.map(|c| {
        ChildCommand {
            cmd: String::from(cmd),
            process: ProcessType::Process(c),
            returned: None,
        }
    });
}