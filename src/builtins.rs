use context::Context;
use context::StdIO;
use nix::errno::Errno;
use std::collections::HashMap;
use std::env;
use std::ffi::CString;
use std::path::Path;

pub type Command = fn(&[CString], &mut Context, StdIO) -> i32;

pub struct Builtins {
    commands: HashMap<CString, Command>
}

impl Builtins {
    pub fn new() -> Builtins {
        let mut b = Builtins {
            commands: HashMap::new()
        };
        b.insert("cd", cd);
        b.insert("export", export);
        b.insert("unset", unset);
        return b;
    }

    fn insert(&mut self, key: &str, val: Command) {
        self.commands.insert(CString::new(key).unwrap(), val);
    }

    pub fn get(&self, key: &CString) -> Option<&Command> {
        self.commands.get(key)
    }
}


fn cd(args: &[CString], context: &mut Context, stdio: StdIO) -> i32 {
    if args.len() > 2 {
        stdio.eprintln(format_args!("rash: too many arguments"));
        return 1;
    }

    let dir = if args.len() == 1 {
        context.env.get("HOME").unwrap_or(String::from("/"))
    } else if args[1].as_bytes() == &[b'-'] {
        if let Some(v) = context.env.get("OLDPWD") {
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
                context.env.set_var("OLDPWD", oldpwd.to_string_lossy().to_string());
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

fn export(args: &[CString], context: &mut Context, stdio: StdIO) -> i32 {
    if args.len() == 1 || args[1] == CString::new("-p").unwrap() {
        context.env.iter()
            .filter(|(_,v)| v.export)
            .for_each(|(k,v)| {
                if let Some(veq) = &v.var_eq {
                    stdio.println(format_args!("export {}", veq.to_string_lossy()));
                } else {
                    stdio.println(format_args!("export {}", k));
                }
            });
        return 0;
    }

    for arg in &args[1..] {
        let arg_str = arg.to_string_lossy().to_string();
        if arg_str.contains("=") {
            let key = context.env.parse_key(arg);
            context.env.export(&key);
            context.env.set_vareq_with_key(key, arg.clone());
        } else {
            context.env.export(&arg_str)
        }
    }

    return 0;
}


fn unset(args: &[CString], context: &mut Context, _stdio: StdIO) -> i32 {
    for arg in &args[1..] {
        let arg_str = arg.to_string_lossy().to_string();
            context.env.unset(&arg_str)
    }

    return 0;
}
