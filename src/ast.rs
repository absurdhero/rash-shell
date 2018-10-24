#[derive(Debug, PartialEq)]
pub struct Program<'a> {
    pub commands: CompleteCommands<'a>
}

#[derive(Debug, PartialEq)]
pub enum CompleteCommands<'a> {
    Command(CompleteCommand<'a>, Box<CompleteCommands<'a>>),
    Nil
}

#[derive(Debug, PartialEq)]
pub enum CompleteCommand<'a> {
    List(TermOp, CommandList<'a>),
    Nil
}

#[derive(Debug, PartialEq)]
pub enum CommandList<'a> {
    AndOr(AndOrCommand<'a>, Box<CommandList<'a>>),
    Nil
}

#[derive(Debug, PartialEq)]
pub enum AndOrCommand<'a> {
    Pipelines(AndOrOp, Pipeline<'a>, Box<AndOrCommand<'a>>),
    Nil
}

#[derive(Debug, PartialEq)]
pub enum AndOrOp {
    And,
    Or,
}

#[derive(Debug, PartialEq)]
pub enum Pipeline<'a> {
    CommandList(Command<'a>, Box<Pipeline<'a>>),
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
