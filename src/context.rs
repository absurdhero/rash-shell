// Copyright 2018 The Rash Project Developers. See the AUTHORS
// file at the top of this distribution for a list of copyright
// holders.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0

use std::fmt;
use std::fs::File;
use std::io::Write;
use std::os::unix::io::FromRawFd;
use std::os::unix::io::IntoRawFd;
use std::os::unix::io::RawFd;

use crate::builtins;
use crate::environment;

/// An evaluation context defines evaluation settings
/// and stores the current shell state.
pub struct Context {
    pub interactive: bool,
    pub last_return: i32,
    pub builtins: builtins::Builtins,
    pub env: environment::Environment,
}

/// encapsulates stdio file descriptors
#[derive(Debug, Copy, Clone)]
pub struct StdIo {
    pub stdin: RawFd,
    pub stdout: RawFd,
    pub stderr: RawFd,
}

impl StdIo {
    pub fn println(&self, fmt: fmt::Arguments) {
        unsafe {
            let mut stdout: File = File::from_raw_fd(self.stdout);
            writeln!(stdout, "{}", fmt).unwrap_or_else(|e| {
                std::process::exit(e.raw_os_error().unwrap_or(74 /* EX_IOERR */))
            });
            stdout.into_raw_fd();
        }
    }

    pub fn eprintln(&self, fmt: fmt::Arguments) {
        unsafe {
            let mut stderr: File = File::from_raw_fd(self.stderr);
            writeln!(stderr, "{}", fmt).unwrap_or_else(|e| {
                std::process::exit(e.raw_os_error().unwrap_or(74 /* EX_IOERR */))
            });
            stderr.into_raw_fd();
        }
    }
}
