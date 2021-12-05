// Copyright 2018 The Rash Project Developers. See the AUTHORS
// file at the top of this distribution for a list of copyright
// holders.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0

use std::collections::HashMap;
use std::env;
use std::path::Path;

use nix::errno::Errno;

use crate::context::Context;
use crate::context::StdIo;

pub type Command = fn(&[String], &mut Context, StdIo) -> i32;

#[derive(Default)]
pub struct Builtins {
    commands: HashMap<String, Command>,
}

impl Builtins {
    pub fn new() -> Builtins {
        let mut b: Builtins = Default::default();
        b.insert("cd", cd);
        b.insert("export", export);
        b.insert("readonly", readonly);
        b.insert("unset", unset);
        b
    }

    fn insert(&mut self, key: &str, val: Command) {
        self.commands.insert(key.to_string(), val);
    }

    pub fn get(&self, key: &str) -> Option<&Command> {
        self.commands.get(key)
    }
}

fn cd(args: &[String], context: &mut Context, stdio: StdIo) -> i32 {
    if args.len() > 2 {
        stdio.eprintln(format_args!("rash: too many arguments"));
        return 1;
    }

    let dir = if args.len() == 1 {
        context.env.get("HOME").unwrap_or("/")
    } else if args[1].as_bytes() == [b'-'] {
        if let Some(v) = context.env.get("OLDPWD") {
            stdio.println(format_args!("{}", v));
            v
        } else {
            stdio.eprintln(format_args!("rash: cd: -: OLDPWD not set"));
            return 1;
        }
    } else {
        &args[1]
    };

    let path = Path::new(&dir);
    let old = env::current_dir();

    match env::set_current_dir(path) {
        Ok(_) => {
            if let Ok(oldpwd) = old {
                context
                    .env
                    .set_var("OLDPWD", oldpwd.to_string_lossy().to_string(), None);
            }

            0
        }
        Err(e) => {
            stdio.eprintln(format_args!(
                "rash: cd: {}: {}",
                path.display(),
                Errno::from_i32(e.raw_os_error().unwrap()).desc()
            ));
            1
        }
    }
}

fn export(args: &[String], context: &mut Context, stdio: StdIo) -> i32 {
    if args.len() == 1 || args[1] == "-p" {
        context
            .env
            .iter()
            .filter(|(_, v)| v.export)
            .for_each(|(k, v)| {
                if let Some(val) = &v.value {
                    stdio.println(format_args!("export {}={}", k, val));
                } else {
                    stdio.println(format_args!("export {}", k));
                }
            });
        return 0;
    }

    for arg in &args[1..] {
        if let Some((key, value)) = context.env.parse(arg) {
            context.env.set_var(key, value.into(), Some(true));
        } else {
            context.env.export(arg);
        }
    }

    0
}

fn readonly(args: &[String], context: &mut Context, stdio: StdIo) -> i32 {
    if args.len() == 1 || args[1] == "-p" {
        context
            .env
            .iter()
            .filter(|(_, v)| v.readonly)
            .for_each(|(k, v)| {
                if let Some(val) = &v.value {
                    stdio.println(format_args!("readonly {}={}", k, val));
                } else {
                    stdio.println(format_args!("readonly {}", k));
                }
            });
        return 0;
    }

    for arg in &args[1..] {
        if let Some((key, value)) = context.env.parse(arg) {
            context.env.set_var(key, value.into(), None);
            context.env.readonly(key);
        } else {
            context.env.readonly(arg);
        }
    }

    0
}

fn unset(args: &[String], context: &mut Context, _stdio: StdIo) -> i32 {
    for arg in &args[1..] {
        context.env.unset(arg)
    }

    0
}
