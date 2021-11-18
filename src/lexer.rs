use std::fmt;
use std::str::CharIndices;

#[derive(Copy, Clone, Debug)]
pub enum Tok<'input> {
    AssignmentWord(&'input str),
    BacktickWord(&'input str),
    BareWord(&'input str),
    DQuoteWord(&'input str),
    SQuoteWord(&'input str),
    Operator(&'input str),
    Newline,
}

impl<'input> fmt::Display for Tok<'input> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Copy, Clone, Debug)]
pub enum LexError<'input> {
    UnexpectedEOF(char),
    Other(&'input str),
}

impl<'input> fmt::Display for LexError<'input> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            LexError::UnexpectedEOF(c) => {
                write!(f, "Unexpected EOF while looking for matching `{}'", c)
            }
            LexError::Other(s) => write!(f, "{}", s),
        }
    }
}

pub type Spanned<Tok, Loc, Error> = Result<(Loc, Tok, Loc), Error>;

pub struct Lexer<'input> {
    chars: std::iter::Peekable<CharIndices<'input>>,
    input: &'input str,
}

impl<'input> Lexer<'input> {
    pub fn new(input: &'input str) -> Self {
        Lexer {
            chars: input.char_indices().peekable(),
            input,
        }
    }

    fn seek_until(&mut self, delim: char, i: usize) -> Option<<Lexer<'input> as Iterator>::Item> {
        loop {
            match self.chars.next() {
                Some((j, c)) if c == delim => {
                    return if i + 1 == j {
                        Some(Ok((i, Tok::DQuoteWord(""), j)))
                    } else {
                        Some(Ok((i, Tok::DQuoteWord(&self.input[i + 1..j]), j)))
                    };
                }
                None => return Some(Err(LexError::UnexpectedEOF(delim))),
                _ => {}
            }
        }
    }

    fn choose_operator(
        &mut self,
        i: usize,
        single: &'input str,
        double: &'input str,
    ) -> Option<<Lexer<'input> as Iterator>::Item> {
        let ret;
        match self.chars.peek() {
            Some((j, p)) if *p == double.chars().nth(1).unwrap() => {
                ret = Some(Ok((i, Tok::Operator(double), *j)));
                self.chars.next();
            }
            _ => ret = Some(Ok((i, Tok::Operator(single), i + 1))),
        };
        return ret;
    }
}

impl<'input> Iterator for Lexer<'input> {
    type Item = Spanned<Tok<'input>, usize, LexError<'input>>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            // TODO: handle backslash escaped characters in all cases, quoted or not

            match self.chars.next() {
                Some((_, ' ')) | Some((_, '\t')) => continue,
                Some((i, '\n')) => return Some(Ok((i, Tok::Newline, i + 1))),
                Some((i, '|')) => return self.choose_operator(i, "|", "||"),
                Some((i, '&')) => return self.choose_operator(i, "&", "&&"),
                Some((i, ';')) => return self.choose_operator(i, ";", ";;"),
                Some((i, '>')) => {
                    let ret;
                    match self.chars.peek() {
                        Some((j, '>')) => {
                            ret = Some(Ok((i, Tok::Operator(">>"), *j)));
                            self.chars.next();
                        }
                        Some((j, '|')) => {
                            ret = Some(Ok((i, Tok::Operator(">|"), *j)));
                            self.chars.next();
                        }
                        Some((j, '&')) => {
                            ret = Some(Ok((i, Tok::Operator(">&"), *j)));
                            self.chars.next();
                        }
                        _ => ret = Some(Ok((i, Tok::Operator(">"), i + 1))),
                    };
                    return ret;
                }
                Some((i, '<')) => {
                    let ret;
                    match self.chars.peek() {
                        Some((j, '>')) => {
                            ret = Some(Ok((i, Tok::Operator("<>"), *j)));
                            self.chars.next();
                        }
                        Some((j, '&')) => {
                            ret = Some(Ok((i, Tok::Operator("<&"), *j)));
                            self.chars.next();
                        }
                        Some((j, '<')) => {
                            let j = *j;
                            self.chars.next();
                            if let Some((k, '-')) = self.chars.peek() {
                                ret = Some(Ok((i, Tok::Operator("<<-"), *k)));
                            } else {
                                ret = Some(Ok((i, Tok::Operator("<<"), j)));
                            }
                        }
                        _ => ret = Some(Ok((i, Tok::Operator("<"), i + 1))),
                    };
                    return ret;
                }
                Some((i, '"')) => {
                    return self.seek_until('"', i);
                }
                Some((i, '\'')) => {
                    return self.seek_until('\'', i);
                }
                Some((i, '`')) => {
                    return self.seek_until('`', i);
                }
                Some((i, _)) => loop {
                    match self.chars.peek() {
                        Some((j, ';')) | Some((j, '|')) | Some((j, '&')) | Some((j, '$'))
                        | Some((j, '"')) | Some((j, '\'')) | Some((j, '`')) | Some((j, ' '))
                        | Some((j, '<')) | Some((j, '>')) => {
                            return Some(Ok((i, Tok::BareWord(&self.input[i..*j]), *j)));
                        }
                        None => {
                            return Some(Ok((
                                i,
                                Tok::BareWord(&self.input[i..]),
                                self.input.len(),
                            )));
                        }
                        _ => {
                            self.chars.next();
                        }
                    }
                },
                None => return None, // End of file
            }
        }
    }
}
