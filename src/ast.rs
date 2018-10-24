#[derive(Debug, PartialEq)]
pub struct Program<'a> {
    pub commands: CompleteCommands<'a>
}

#[derive(Debug, PartialEq)]
pub enum CompleteCommands<'a> {
    CompleteCommands(CompleteCommand<'a>, Box<CompleteCommands<'a>>),
    Nil
}

#[derive(Debug, PartialEq)]
pub enum CompleteCommand<'a> {
    CompleteCommand(TermOp, List<'a>),
    Nil
}

#[derive(Debug, PartialEq)]
pub enum List<'a> {
    List(TermOp, AndOr<'a>, Box<List<'a>>),
    Nil,
}

#[derive(Debug, PartialEq)]
pub enum AndOr<'a> {
    AndOr(AndOrOp, Pipeline<'a>, Box<AndOr<'a>>),
    Nil,
}

#[derive(Debug, PartialEq)]
pub enum AndOrOp {
    And,
    Or,
}

#[derive(Debug, PartialEq)]
pub enum Pipeline<'a> {
    Pipeline(Command<'a>, Box<Pipeline<'a>>),
    Not(Box<Pipeline<'a>>),
    Nil,
}

#[derive(Debug, PartialEq)]
pub enum Command<'a> {
    Simple {
        assign: Arg<'a>,
        args: Arg<'a>,
        //redirect: Vec<Redirect<'a>>,
    },
}

#[derive(Debug, PartialEq)]
pub enum Arg<'a> {
    CmdWord(&'a str, Box<Arg<'a>>),
    Arg(&'a str, Box<Arg<'a>>),
    Backquote(Box<Arg<'a>>),
    Nil,
}

#[derive(Debug, PartialEq)]
pub enum TermOp {
    Semi,
    Amp
}

//pub struct Redirect<'a> {
//    command: &'a Command<'a>,
//    operator: RedirectionType,
//    fname: Arg<'a>,
//    fd: int,
//    dupfd: int,
//}

//pub enum RedirectionType {
//    TO,     // fd > fname
//    CLOBBER,// fd >| fname
//    FROM,   // fd < fname
//    FROMTO, // fd <> fname
//    APPEND, // fd >> fname
//    TOFD,   // fd <& dupfd
//    FROMFD, // fd >& dupfd
//}


#[cfg(test)]
#[allow(unused_imports)]
mod tests {
    use super::*;
    use grammar;

    #[test]
    fn valid_commands() {
        let parser = grammar::programParser::new();

        assert!(parser.parse("test\n").is_ok());
        assert!(parser.parse("test foo &\n").is_ok());

        assert!(parser.parse("test | | \n").is_err());
    }
}
