use std::env;
use std::ffi::CString;
use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;
use std::path::PathBuf;

use nix::unistd::*;
use void::Void;

use context;

static ENOENT: nix::Error = nix::Error::Sys(nix::errno::Errno::ENOENT);

/// executes a command and returns the Pid if a child was forked or None if a built-in was called.
pub fn run_command(context: &mut context::Context,
                   cmd: &CString,
                   args: &[CString],
                   env: &[CString],
                   stdio: context::StdIO) -> Option<Pid> {
    let maybe_builtin;
    {
        maybe_builtin = context.builtins.get(cmd).map(|c| *c)
    }

    if let Some(c) = maybe_builtin {
        let ret: Option<i32>;
        {
            ret = Some(c(args, context, stdio));
        }
        context.last_return = ret;
        if stdio.stdin != 0 { close(stdio.stdin).unwrap(); }
        if stdio.stdout != 1 { close(stdio.stdout).unwrap(); }
        return None;
    }

    match fork() {
        Ok(ForkResult::Parent { child }) => {
            if stdio.stdin != 0 { close(stdio.stdin).unwrap(); }
            if stdio.stdout != 1 { close(stdio.stdout).unwrap(); }
            return Some(child);
        }
        Ok(ForkResult::Child) => {
            dup2(stdio.stdin, 0).expect("could not dup stdin");
            dup2(stdio.stdout, 1).expect("could not dup stdout");
            // wire up stdin from last thing in pipeline and exec
            if let Err(e) = exec(context, cmd, args, env) {
                println!("could not exec: {}", e);
                close(stdio.stdin).unwrap();
                close(stdio.stdout).unwrap();
            }
        }
        Err(_) => println!("rash: fork failed"),
    }
    return None;
}


/// search for the filename in the PATH and try to exec until one succeeds
pub fn exec(context: &context::Context, filename: &CString, args: &[CString], env: &[CString]) -> nix::Result<Void> {
    // add any prefixed variables to the environment
    let mut child_env = context.env.clone();
    for v in env.iter() {
        child_env.set_vareq(v.to_owned());
    }

    let path = child_env.get("PATH");
    let exported = child_env.into_exported();

    // if the filename has any slashes in it, don't search the PATH
    if filename.as_bytes().iter().any(|c| *c == b'/') {
        return try_exec(filename, args, &exported);
    }

    // if matching paths are found but none of them can be executed, return the error
    // from the first attempt.
    // if no matches are found, return ENOENT.

    let mut first_error: nix::Error = ENOENT;

    match path {
        Some(paths) => {
            for path in env::split_paths(&paths) {
                if let Err(mut e) = try_exec(&filepath(path, filename), args, &exported) {
                    if first_error == ENOENT {
                        first_error = e;
                    }
                }
            }
        }
        None => {
            for path in vec!["/bin", "/usr/bin"] {
                let path_buf = PathBuf::from(path);
                if let Err(mut e) = try_exec(&filepath(path_buf, filename), args, &exported) {
                    if first_error == ENOENT {
                        first_error = e;
                    }
                }
            }
        }
    }

    return Err(first_error);
}

fn try_exec(filepath: &CString, args: &[CString], exported_env: &[CString]) -> nix::Result<Void> {
    execve(filepath, args, exported_env)
}

fn filepath(path: PathBuf, filename: &CString) -> CString {
    // If I understand what I just wrote, this creates two copies of the data.
    // join makes a new OsString and String::from copies the data into a new String.
    // I don't yet understand the best way to go from Path -> PathBuf -> &str
    let path_buf = path.join(OsStr::from_bytes(filename.as_bytes()));
    CString::new(path_buf.into_os_string().to_str().unwrap()).unwrap()
}
