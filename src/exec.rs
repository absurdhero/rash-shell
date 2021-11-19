// Copyright 2018 The Rash Project Developers. See the AUTHORS
// file at the top of this distribution for a list of copyright
// holders.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0

use std::convert::Infallible;
use std::env;
use std::ffi::CString;
use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;
use std::path::PathBuf;

use nix::unistd::*;

use crate::context;

static ENOENT: nix::Error = nix::Error::Sys(nix::errno::Errno::ENOENT);

/// executes a command and returns the Pid if a child was forked or None if a built-in was called.
pub fn run_command(
    context: &mut context::Context,
    cmd: &str,
    args: &[String],
    env: &[String],
    stdio: context::StdIo,
) -> Option<Pid> {
    let maybe_builtin;
    {
        maybe_builtin = context.builtins.get(cmd).copied()
    }

    if let Some(c) = maybe_builtin {
        let ret: i32;
        {
            ret = c(args, context, stdio);
        }
        context.last_return = ret;
        if stdio.stdin != 0 {
            close(stdio.stdin).unwrap();
        }
        if stdio.stdout != 1 {
            close(stdio.stdout).unwrap();
        }
        return None;
    }

    match unsafe { fork() } {
        Ok(ForkResult::Parent { child }) => {
            if stdio.stdin != 0 {
                close(stdio.stdin).unwrap();
            }
            if stdio.stdout != 1 {
                close(stdio.stdout).unwrap();
            }
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
    None
}

/// search for the filename in the PATH and try to exec until one succeeds
pub fn exec(
    context: &context::Context,
    filename: &str,
    args: &[String],
    env: &[String],
) -> nix::Result<Infallible> {
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
                if let Err(e) = try_exec(&filepath(path, filename), args, &exported) {
                    if first_error == ENOENT {
                        first_error = e;
                    }
                }
            }
        }
        None => {
            for path in &["/bin", "/usr/bin"] {
                let path_buf = PathBuf::from(path);
                if let Err(e) = try_exec(&filepath(path_buf, filename), args, &exported) {
                    if first_error == ENOENT {
                        first_error = e;
                    }
                }
            }
        }
    }

    Err(first_error)
}

fn try_exec(filepath: &str, args: &[String], exported_env: &[String]) -> nix::Result<Infallible> {
    let arg_cstrings = args
        .iter()
        .map(|c| CString::new(c.as_str()).unwrap())
        .collect::<Vec<_>>();

    let exported_cstrings = exported_env
        .iter()
        .map(|c| CString::new(c.as_str()).unwrap())
        .collect::<Vec<_>>();

    execve(
        CString::new(filepath).unwrap().as_c_str(),
        arg_cstrings
            .iter()
            .map(|c| c.as_c_str())
            .collect::<Vec<_>>()
            .as_slice(),
        exported_cstrings
            .iter()
            .map(|c| c.as_c_str())
            .collect::<Vec<_>>()
            .as_slice(),
    )
}

fn filepath(path: PathBuf, filename: &str) -> String {
    let path_buf = path.join(OsStr::from_bytes(filename.as_bytes()));
    // assumes the path is valid UTF-8
    path_buf.into_os_string().into_string().unwrap()
}
