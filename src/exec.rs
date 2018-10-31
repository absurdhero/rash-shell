use nix::unistd::*;
use std::env;
use std::ffi::CString;
use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;
use std::path::PathBuf;
use void::Void;

static ENOENT: nix::Error = nix::Error::Sys(nix::errno::Errno::ENOENT);

/// search for the filename in the PATH and try to exec until one succeeds
pub fn exec(filename: &CString, args: &[CString], env: &[CString]) -> nix::Result<Void> {
    // if the filename has any slashes in it, don't search the PATH
    if filename.as_bytes().iter().any(|c| *c == b'/') {
        return try_exec(filename, args, env);
    }

    // if matching paths are found but none of them can be executed, return the error
    // from the first attempt.
    // if no matches are found, return ENOENT.

    let mut first_error: nix::Error = ENOENT;

    match env::var_os("PATH") {
        Some(paths) => {
            for path in env::split_paths(&paths) {
                if let Err(mut e) =try_exec(&filepath(path, filename), args, env) {
                    if first_error == ENOENT {
                        first_error = e;
                    }
                }
            }
        }
        None => {
            for path in vec!["/bin", "/usr/bin"] {
                let path_buf = PathBuf::from(path);
                if let Err(mut e) = try_exec(&filepath(path_buf, filename), args, env) {
                    if first_error == ENOENT {
                        first_error = e;
                    }
                }
            }
        }
    }

    return Err(first_error);
}

fn try_exec(filepath: &CString, args: &[CString], env: &[CString]) -> nix::Result<Void> {
    execve(filepath, args, env)
}

fn filepath(path: PathBuf, filename: &CString) -> CString {
    // If I understand what I just wrote, this creates two copies of the data.
    // join makes a new OsString and String::from copies the data into a new String.
    // I don't yet understand the best way to go from Path -> PathBuf -> &str
    let path_buf = path.join(OsStr::from_bytes(filename.as_bytes()));
    CString::new(path_buf.into_os_string().to_str().unwrap()).unwrap()
}
