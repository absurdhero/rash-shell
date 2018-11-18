// Copyright 2018 The Rash Project Developers. See the AUTHORS
// file at the top of this distribution for a list of copyright
// holders.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0

use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::env;
use std::ffi::CString;

#[derive(Debug, Clone, Default)]
pub struct Val {
    pub var_eq: Option<CString>,
    pub export: bool,
    pub readonly: bool,
}

#[derive(Debug, Clone)]
pub struct Environment {
    vars: HashMap<String, Val>
}

impl Environment {
    pub fn set_var(&mut self, key: &str, val: String) {
        self.set_vareq_with_key(key.to_string(), CString::new(format!("{}={}", key, val)).unwrap());
    }

    /// sets a variable of the form "KEY=VALUE"
    pub fn set_vareq(&mut self, var_eq: CString) {
        let key: String = self.parse_key(&var_eq);
        self.set_vareq_with_key(key, var_eq);
    }

    pub fn set_vareq_with_key(&mut self, key: String, var_eq: CString) {
        match self.vars.entry(key) {
            Entry::Occupied(mut o) => {
                let v = o.get_mut();
                // TODO: return an error if readonly
                if !v.readonly {
                    v.var_eq = Some(var_eq);
                }
            }
            Entry::Vacant(o) => {
                o.insert(Val { var_eq: Some(var_eq), ..Default::default() });
            }
        }
    }

    pub fn unset(&mut self, key: &str) {
        if let Entry::Occupied(o) = self.vars.entry(key.to_string()) {
            // TODO: return an error if readonly
            if !o.get().readonly {
                o.remove();
            }
        }
    }

    pub fn export(&mut self, key: &str) {
        self.vars.entry(key.to_string())
            .or_insert(Val { ..Default::default() })
            .export = true;
    }

    pub fn readonly(&mut self, key: &str) {
        self.vars.entry(key.to_string())
            .or_insert(Val { ..Default::default() })
            .readonly = true;
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
