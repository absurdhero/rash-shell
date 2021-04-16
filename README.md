# rash - the Rust Bourne Shell

[![Build Status](https://travis-ci.org/absurdhero/rash-shell.svg?branch=master)](https://travis-ci.org/absurdhero/rash-shell)
![Crates.io](https://img.shields.io/crates/v/rash-shell.svg)

A bourne shell inspired by dash.

## Goals

These goals guide design decisions and the feature roadmap.
Once the shell is complete, we can look at more ambitious goals like
implementing features beyond POSIX, extending the programming language,
and more aggressive speed optimizations.

The guiding goals include:

 - A high level of POSIX compliance
 - Helpful error messages
 - Modern terminal interface features
 - Fast
 - Secure by default (even when it means violating POSIX)

## Design

The overall design is inspired by the dash shell which is the default
`/bin/sh` on Debian and Ubuntu. It has a long lineage dating back to
early bourne shells.

Dash was built for speed, POSIX compliance, and a small codebase.
It's a great source of design wisdom and serves as a good reference
implementation when the POSIX spec is unclear.

Rash's parser and evaluator take inspiration from dash.
Rash uses a parser generator called [LALRPOP](https://github.com/lalrpop/lalrpop)
to generate an AST. It then walks the AST evaluating statements.
The grammar and AST are based on the grammar and terminology defined by
POSIX which makes it easy to use the standard as a reference for
understanding rash.

## Contributions

Contributions are welcomed! The status list below is a great place
to find a feature to implement. If the codebase looks too complicated
at first, there are several features that don't require changing much
existing code.

Open a GitHub Issue for questions, requests, or patches.

The project is licensed under the Apache License 2.0.

## Status

Except for the most obscure features,
syntax and functionality are either fully implemented or not at all.
This means that you can be confident when using the features marked as
complete below. Any errors or omissions in existing functionality are
bugs and may be treated as such.

Features:

- [x] executing commands
- [x] pipelines
- [x] boolean logic
- [x] async execution (`&`)
- [x] path searching
- [x] variable interpolation
- [x] environment manipulation (export, unset, readonly, variable prefixes)
- [x] cd
- [x] saving the exit status (`$?`)
- [ ] subshells
- [ ] interpolating $() and backticks
- [ ] eval quotes, single quotes
- [ ] I/O Redirection
- [ ] job control
- [ ] shell startup arguments (e.g. `-c`, `-l`)
- [ ] `set` command
- [ ] control flow operators (`if`, `while`, `case`)

Interactive Mode:

- [x] history (not yet configurable or persistent)
- [x] keybindings (not yet configurable)
- [ ] configurable settings
- [ ] tab completion

Quality Improvements:

- [ ] Compliance test suite
    - See [osh tests](http://www.oilshell.org/cross-ref.html?tag=spec-test#spec-test)
- [ ] consistent error message format
- [ ] Path hashing
- [ ] Other common performance optimizations

# References

[dash homepage](http://gondor.apana.org.au/~herbert/dash/)

[POSIX Shell Grammar](http://pubs.opengroup.org/onlinepubs/9699919799/utilities/V3_chap02.html#tag_18_10_02)

[Survey of CLI shell features](https://en.wikipedia.org/wiki/Comparison_of_command_shells)

[ion shell (written in rust)](https://gitlab.redox-os.org/redox-os/ion)

[oursh (written in rust)](https://github.com/nixpulvis/oursh)

# Relevant Rust Language Information

[Discussion about looping over recursive structures](https://stackoverflow.com/questions/37986640/cannot-obtain-a-mutable-reference-when-iterating-a-recursive-structure-cannot-b)
