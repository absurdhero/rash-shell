use builtins;
use environment;
use std::fs::File;
use std::io::Write;
use std::os::unix::io::FromRawFd;
use std::os::unix::io::IntoRawFd;
use std::os::unix::io::RawFd;
use std::fmt;

/// An evaluation context defines evaluation settings
/// and stores the current shell state.
pub struct Context {
    pub interactive: bool,
    pub last_return: Option<i32>,
    pub builtins: builtins::Builtins,
    pub env: environment::Environment,
}


/// encapsulates stdio file descriptors
#[derive(Debug, Copy, Clone)]
pub struct StdIO {
    pub stdin: RawFd,
    pub stdout: RawFd,
    pub stderr: RawFd,
}

impl StdIO {
    pub fn println(&self, fmt: fmt::Arguments) {
        unsafe {
            let mut stdout: File = File::from_raw_fd(self.stdout);
            writeln!(stdout, "{}", fmt);
            stdout.into_raw_fd();
        }
    }

    pub fn eprintln(&self, fmt: fmt::Arguments) {
        unsafe {
            let mut stderr: File = File::from_raw_fd(self.stderr);
            writeln!(stderr, "{}", fmt);
            stderr.into_raw_fd();
        }
    }
}
