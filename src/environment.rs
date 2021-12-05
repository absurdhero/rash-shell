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

#[derive(Debug, Clone, Default)]
pub struct Val {
    pub value: Option<String>,
    pub export: bool,
    pub readonly: bool,
}

#[derive(Debug, Clone)]
pub struct Environment {
    vars: HashMap<String, Val>,
}

impl Environment {
    pub fn set_var(&mut self, key: &str, val: String, export: Option<bool>) {
        match self.vars.entry(key.into()) {
            Entry::Occupied(mut o) => {
                let v = o.get_mut();
                // TODO: return an error if readonly
                if !v.readonly {
                    v.value = Some(val);
                    v.export = export.unwrap_or(false);
                }
            }
            Entry::Vacant(o) => {
                o.insert(Val {
                    value: Some(val),
                    export: export.unwrap_or(false),
                    ..Default::default()
                });
            }
        }
    }

    /// sets a variable of the form "KEY=VALUE"
    pub fn set_vareq(&mut self, var_eq: &str) {
        if let Some((key, value)) = self.parse(var_eq) {
            self.set_var(key, value.into(), None)
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
        self.vars
            .entry(key.to_string())
            .or_insert(Val {
                ..Default::default()
            })
            .export = true;
    }

    pub fn readonly(&mut self, key: &str) {
        self.vars
            .entry(key.to_string())
            .or_insert(Val {
                ..Default::default()
            })
            .readonly = true;
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        let val = self.vars.get(key)?;
        val.value.as_deref()
    }

    pub fn exports(&self) -> Vec<String> {
        self.vars
            .iter()
            .filter(|(_, v)| v.export && v.value.is_some())
            .map(|(k, v)| format!("{}={}", k, v.value.as_ref().unwrap()))
            .collect()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &Val)> {
        self.vars.iter()
    }

    pub fn parse<'a>(&self, var_eq: &'a str) -> Option<(&'a str, &'a str)> {
        if var_eq.len() < 2 {
            return None;
        }
        // variable names can start with '=' according to glibc and rust's environment parsing
        // but POSIX doesn't allow it. We'll err on the side of being more accepting for now.
        for (i, c) in var_eq[1..].char_indices() {
            if c == '=' {
                let i = i + 1; // compensate for scanning from the 2nd position
                return Some((&var_eq[..i], &var_eq[i + 1..]));
            }
        }
        None
    }
}

pub fn empty() -> Environment {
    Environment {
        vars: HashMap::new(),
    }
}

pub fn from_system() -> Environment {
    let mut e = empty();
    env::vars().for_each(|(k, v)| {
        e.set_var(&k, v, Some(true));
    });
    e
}
