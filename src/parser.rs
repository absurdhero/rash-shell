extern crate std;

use std::str;
use nom::IResult;
use nom::{is_space, newline};

#[derive(Debug, PartialEq)]
pub struct Command<'a> {
    argv: Vec<&'a str>,
}
impl<'a> Command<'a> {
    pub fn command(&self) -> &str {
        self.argv[0]
    }

    pub fn argv(&self) -> &Vec<&str> {
        &self.argv
    }
}

named!(arg_list<&[u8], Vec<&str>>,
    separated_list!(
        char!(' '),
        map_res!(
            take_till!(|c| { is_space(c) || c == b'\n' }),
            std::str::from_utf8))
);

named!(command<&[u8], Command>,
    do_parse!(
        argv: arg_list >>
        newline >>
        (Command { argv })
    )
);

pub fn parse(input: &[u8]) -> IResult<&[u8], Command, u32> {
    command(input)
}

pub fn parse_str(input: &str) -> IResult<&[u8], Command, u32> {
    command(input.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;
    use nom::Err as nomErr;

    #[test]
    fn command_with_no_arg() {
        match parse_str("cmd\n") {
            Ok((_, cmd)) => {
                assert_eq!(cmd, Command {
                    argv: vec!["cmd"],
                });
            }
            e => panic!("{:?}", e),
        }
    }

    #[test]
    fn command_with_one_arg() {
        match parse_str("first second\n") {
            Ok((_, cmd)) => {
                assert_eq!(cmd, Command {
                    argv: vec!["first", "second"],
                });
            }
            e => panic!("{:?}", e),
        }
    }

    #[test]
    fn incomplete_when_line_has_no_ending() {
        match parse_str("no newline yet") {
            Err(nomErr::Incomplete(_)) => (),
            Ok(_) => panic!("expected incomplete parse"),
            _ => panic!("expected a different kind of error"),
        }
    }
}
