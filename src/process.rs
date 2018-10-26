use std::collections::HashMap;
use std::process::*;


pub struct ChildCommand {
    pub cmd: String,
    pub process: ProcessType,
    pub returned: Option<i32>,
}

pub enum ProcessType {
    Process(Child),
    Builtin,
}

pub fn exec(env: &HashMap<String, String>, cmd: &str, args: &Vec<&str>) -> std::io::Result<ChildCommand> {
    let res = Command::new(cmd)
        .envs(env)
        .args(args)
        .stdin(Stdio::piped())
        .spawn();
    return res.map(|c| {
        ChildCommand {
            cmd: String::from(cmd),
            process: ProcessType::Process(c),
            returned: None,
        }
    });
}