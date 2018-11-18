use std::collections::HashMap;
use std::env;
use std::ffi::CString;

#[derive(Debug, Clone)]
pub struct Val {
    pub var_eq: Option<CString>,
    pub export: bool,
}

#[derive(Debug, Clone)]
pub struct Environment {
    // map from a string key to a key=value formatted CString
    vars: HashMap<String, Val>
}

impl Environment {
    pub fn set_var(&mut self, key: &str, val: String) {
        self.vars.insert(key.to_string(), Val { var_eq: Some(CString::new(format!("{}={}", key, val)).unwrap()), export: false });
    }

    /// sets a variable of the form "KEY=VALUE"
    pub fn set_vareq(&mut self, var_eq: CString) {
        let key: String = self.parse_key(&var_eq);
        self.set_vareq_with_key(key, var_eq);
    }

    pub fn set_vareq_with_key(&mut self, key: String, var_eq: CString) {
        self.vars.entry(key)
            .or_insert(Val { var_eq: None, export: false })
            .var_eq = Some(var_eq);
    }

    pub fn unset(&mut self, key: &str) {
        self.vars.remove(key);
    }

    pub fn export(&mut self, key: &str) {
        self.vars.entry(key.to_string())
            .or_insert(Val { var_eq: None, export: true })
            .export = true;
    }

    pub fn get(&self, key: &str) -> Option<String> {
        let entry = self.vars.get(key);
        if entry.is_none() {
            return None;
        }

        if let Some(var_eq) = &entry.unwrap().var_eq {
            let mut split = var_eq.as_bytes().split(|b| *b == b'=');
            let _entry_key = split.next().unwrap();
            if let Some(value) = split.next() {
                return Some(String::from_utf8_lossy(value).to_string());
            } else {
                return Some(String::new());
            }
        } else {
            return None;
        }
    }

    pub fn into_exported(self) -> Vec<CString> {
        self.vars.into_iter()
            .filter(|(_, v)| v.export && v.var_eq.is_some())
            .map(|(_, v)| v.var_eq.unwrap())
            .collect()
    }

    pub fn iter(&self) -> impl Iterator<Item=(&String, &Val)> {
        self.vars.iter()
    }

    pub fn parse_key(&self, var_eq: &CString) -> String {
        let mut split = var_eq.as_bytes().split(|b| *b == b'=');
        String::from_utf8_lossy(split.next().unwrap()).to_string()
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
