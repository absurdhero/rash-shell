use std::fmt;
use std::str::CharIndices;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum TokType {
    AssignmentWord,
    Word,
    Operator,
    EOF,
}

impl fmt::Display for TokType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Tok<'input> {
    pub tok_type: TokType,
    pub input: &'input str,
}

impl<'input> fmt::Display for Tok<'input> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl<'input> Tok<'input> {
    fn new(tok_type: TokType, input: &'input str) -> Tok<'input> {
        Tok { tok_type, input }
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
    cur_type: TokType,
    cur_start: usize,
    past_first_word: bool,
    next: Option<(usize, char)>,
}

impl<'input> Lexer<'input> {
    pub fn new(input: &'input str) -> Self {
        let mut chars = input.char_indices().peekable();
        let next = chars.next();

        Lexer {
            chars,
            input,
            cur_type: TokType::EOF,
            cur_start: 0,
            past_first_word: false,
            next,
        }
    }

    fn is_operator_start(c: char) -> bool {
        c == ';' || c == '|' || c == '&' || c == '<' || c == '>'
    }

    fn is_operator(s: &str) -> bool {
        if s.len() == 1 && Lexer::is_operator_start(s.chars().next().unwrap()) {
            return true;
        }

        s == "&&"
            || s == "||"
            || s == ";;"
            || s == "<<"
            || s == ">>"
            || s == "<&"
            || s == ">&"
            || s == "<>"
            || s == "<<-"
            || s == ">|"
    }

    fn delimit(&mut self, end: usize) -> Option<Spanned<Tok<'input>, usize, LexError<'input>>> {
        // elide empty tokens
        if self.cur_type == TokType::EOF {
            return None;
        }

        let start = self.cur_start;

        if self.cur_type == TokType::Word && !self.past_first_word {
            // check if this word qualifies as an assignment word
            let word = &self.input[start..end];
            let name_idx = word.find('=');
            if name_idx == None || name_idx == Some(0) || !is_name(&word[0..name_idx.unwrap()]) {
                // once we stop finding assignment words, we're done for good.
                self.past_first_word = true;
            } else {
                self.cur_type = TokType::AssignmentWord;
            }
        }

        let t = (start, Tok::new(self.cur_type, &self.input[start..end]), end);

        // reset token state
        self.cur_start = end;
        self.cur_type = TokType::EOF;

        Some(Ok(t))
    }
}

impl<'input> Iterator for Lexer<'input> {
    type Item = Spanned<Tok<'input>, usize, LexError<'input>>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut continued = false;
        let mut quoted: Option<char> = None;
        let mut slash_escaped = false;

        loop {
            // don't advance the character on the first iteration. We may still be scanning
            // a character from the previous call to next() that wasn't consumed.
            if continued {
                self.next = self.chars.next();
            }
            continued = true;

            match self.next {
                Some((i, c)) => {
                    if quoted.is_none() && !slash_escaped {
                        if self.cur_type == TokType::Operator {
                            if Lexer::is_operator(&self.input[self.cur_start..=i]) {
                                continue;
                            } else {
                                return self.delimit(i);
                            }
                        }

                        // unquoted spaces delimit the current token
                        if c == ' ' || c == '\t' {
                            if let Some(s) = self.delimit(i) {
                                self.cur_start = i + 1;
                                return Some(s);
                            }
                            // if there was no token before the space, keep going.
                            self.cur_start = i + 1; // skip whitespace
                            continue;
                        }

                        if c == '"' || c == '\'' {
                            quoted = Some(c);
                        } else if c == '\\' {
                            slash_escaped = true;
                        }

                        if c == '`' {
                            quoted = Some(c);
                        }
                    } else if slash_escaped {
                        // immediately end escaping and continue scanning
                        slash_escaped = false;
                        continue;
                    } else if quoted == Some('\'') {
                        if c == '\'' {
                            quoted = None;
                        }
                        continue;
                    } else if quoted == Some('`') {
                        if c == '`' {
                            quoted = None;
                        }
                        continue;
                    } else if quoted == Some('"') {
                        if c == '"' {
                            quoted = None;
                        } else if c == '\\' {
                            slash_escaped = true;
                        }
                        continue;
                    }

                    if Lexer::is_operator_start(c) {
                        if let Some(s) = self.delimit(i) {
                            self.cur_type = TokType::Operator;
                            return Some(s);
                        }
                        // if there was no token before the operator, keep going.
                        self.cur_type = TokType::Operator;
                        self.cur_start = i;
                        continue;
                    }

                    // anything else is part of a word, keep reading the token.
                    self.cur_type = TokType::Word;
                }
                None => {
                    return if self.cur_type == TokType::EOF {
                        None // EOF
                    } else {
                        if let Some(q) = quoted {
                            return Some(Err(LexError::UnexpectedEOF(q)));
                        }
                        return self.delimit(self.input.len());
                    };
                }
            }
        }
    }
}

// 3.230 Name: An alphanumeric word that does not begin with a digit
fn is_name(s: &str) -> bool {
    let first = s.chars().next();
    if first == None || first.unwrap().is_ascii_digit() {
        false
    } else {
        s.chars().all(|c| c.is_alphanumeric() || c == '_')
    }
}
