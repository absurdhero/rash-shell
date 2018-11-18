use std::collections::HashMap;
use std::env;
use std::ffi::CString;

#[derive(Debug, Clone)]
struct Val {
    var_eq: CString,
    export: bool,
}

#[derive(Debug, Clone)]
pub struct Environment {
    // map from a string key to a key=value formatted CString
    vars: HashMap<String, Val>
}

impl Environment {
    pub fn set_var(&mut self, key: &str, val: String) {
        self.vars.insert(key.to_string(), Val { var_eq: CString::new(format!("{}={}", key, val)).unwrap(), export: false });
    }

    pub fn set_vareq(&mut self, var_eq: CString) {
        let key: String;
        {
            let mut split = var_eq.as_bytes().split(|b| *b == b'=');
            key = String::from_utf8_lossy(split.next().unwrap()).to_string();
        }
        self.vars.insert(key, Val { var_eq, export: false });
    }

    pub fn unset_var(&mut self, key: &str) {
        self.vars.remove(key);
    }

    pub fn export(&mut self, key: &str) {
        self.vars.get_mut(key).map(|v| v.export = true);
    }

    pub fn get(&self, key: &str) -> Option<String> {
        let entry = self.vars.get(key);
        if entry.is_none() {
            return None;
        }

        let var_eq = entry.unwrap();
        let mut split = var_eq.var_eq.as_bytes().split(|b| *b == b'=');
        let _entry_key = split.next().unwrap();
        if let Some(value) = split.next() {
            return Some(String::from_utf8_lossy(value).to_string());
        } else {
            return Some(String::new());
        }
    }

    pub fn into_exported(self) -> Vec<CString> {
        self.vars.into_iter()
            .filter(|(_, v)| v.export)
            .map(|(_, v)| v.var_eq)
            .collect()
    }
}

pub fn empty() -> Environment {
    return Environment { vars: HashMap::new() };
}

pub fn from_system() -> Environment {
    let mut e = empty();
    env::vars().for_each(|(k, v)| {
        e.set_var(&k, v);
        e.export(&k);
    });
    return e;
}
