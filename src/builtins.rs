use context::StdIO;
use nix::errno::Errno;
use std::collections::HashMap;
use std::env;
use std::ffi::CString;
use std::path::Path;

type Command = fn(&[CString], StdIO) -> i32;

pub struct Builtins {
    commands: HashMap<CString, Command>
}

impl Builtins {
    pub fn new() -> Builtins {
        let mut b = Builtins {
            commands: HashMap::new()
        };
        b.insert("cd", cd);
        return b;
    }

    fn insert(&mut self, key: &str, val: Command) {
        self.commands.insert(CString::new(key).unwrap(), val);
    }

    pub fn get(&self, key: &CString) -> Option<&Command> {
        self.commands.get(key)
    }
}


fn cd(args: &[CString], stdio: StdIO) -> i32 {
    if args.len() > 2 {
        stdio.eprintln(format_args!("rash: too many arguments"));
        return 1;
    }

    let dir = if args.len() == 1 {
        env::var("HOME").unwrap_or(String::from("/"))
    } else if args[1].as_bytes() == &[b'-'] {
        if let Ok(v) = env::var("OLDPWD") {
            stdio.println(format_args!("{}", v));
            v
        } else {
            stdio.eprintln(format_args!("rash: cd: -: OLDPWD not set"));
            return 1;
        }
    } else {
        String::from_utf8(args[1].as_bytes().to_vec()).unwrap()
    };

    let path = Path::new(&dir);
    let old = env::current_dir();

    match env::set_current_dir(path) {
        Ok(_) => {
            if let Ok(oldpwd) = old {
                env::set_var("OLDPWD", oldpwd);
            }

            return 0;
        }
        Err(e) => {
            stdio.eprintln(
                format_args!(
                    "rash: cd: {}: {}",
                    path.display(),
                    Errno::from_i32(e.raw_os_error().unwrap()).desc()));
            return 1;
        }
    }
}

