#[derive(Debug, PartialEq)]
pub struct Program<'a> {
    pub commands: CompleteCommands<'a>
}

#[derive(Debug, PartialEq)]
pub struct CompleteCommands<'a> {
    pub complete_commands: Vec<CompleteCommand<'a>>,
}

impl<'a> CompleteCommands<'a> {
    pub fn push(mut self: CompleteCommands<'a>, element: CompleteCommand<'a>) -> CompleteCommands<'a> {
        self.complete_commands.push(element);
        self
    }
}

#[derive(Debug, PartialEq)]
pub struct CompleteCommand<'a> {
    pub and_ors: Vec<(TermOp, AndOr<'a>)>,
}

impl<'a> CompleteCommand<'a> {
    pub fn push(mut self, op: TermOp, element: AndOr<'a>) -> CompleteCommand<'a> {
        // update the TermOp of the previous list entry
        self.update_last(op);
        // add the new entry and assume it ends with a semicolon
        self.and_ors.push((TermOp::Semi, element));
        self
    }

    pub fn update_last(&mut self, op: TermOp) {
        if let Some((_, e)) = self.and_ors.pop() {
            self.and_ors.push((op, e));
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct AndOr<'a> {
    pub pipelines: Vec<(AndOrOp, Pipeline<'a>)>,
}

impl<'a> AndOr<'a> {
    pub fn push(mut self, op: AndOrOp, element: Pipeline<'a>) -> AndOr<'a> {
        if let Some((_, e)) = self.pipelines.pop() {
            self.pipelines.push((op, e));
        }
        self.pipelines.push((AndOrOp::And, element));
        self
    }
}

#[derive(Debug, PartialEq)]
pub enum AndOrOp {
    And,
    Or,
}

#[derive(Debug, PartialEq)]
pub struct Pipeline<'a> {
    pub commands: Vec<Command<'a>>,
    pub negated: bool,
}

impl<'a> Pipeline<'a> {
    pub fn new(cmd: Command<'a>) -> Pipeline<'a> {
        Pipeline { commands: vec![cmd], negated: false }
    }

    pub fn negate(mut self) -> Pipeline<'a> {
        self.negated = !self.negated;
        self
    }

    pub fn push(mut self, cmd: Command<'a>) -> Pipeline<'a> {
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
