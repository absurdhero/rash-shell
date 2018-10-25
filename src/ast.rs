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
pub struct Pipeline<'a> {
    commands: Vec<Command<'a>>,
    negated: bool,
}

impl<'a> Pipeline<'a> {
    pub fn new(cmd: Command<'a>) -> Pipeline<'a> {
        Pipeline { commands: vec![cmd], negated: false }
    }

    pub fn negate(mut self) -> Pipeline<'a> {
        self.negated = !self.negated;
        self
    }

    pub fn push(mut self: Pipeline<'a>, cmd: Command<'a>) -> Pipeline<'a> {
        self.commands.push(cmd);
        self
    }
}

#[derive(Debug, PartialEq)]
pub enum Command<'a> {
    Simple {
        assign: Vec<Arg<'a>>,
        cmd: Arg<'a>,
        args: Vec<Arg<'a>>,
        //redirect: Vec<Redirect<'a>>,
    },
}

#[derive(Debug, PartialEq)]
pub enum Arg<'a> {
    Arg(&'a str),
    Backquote(Vec<Arg<'a>>),
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
