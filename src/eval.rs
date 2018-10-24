use ast;

pub fn eval(program: &mut ast::Program) -> () {
    let mut complete_commands = &mut program.commands;
    loop {
        match {complete_commands} {
            ast::CompleteCommands::Command(cc, rest) => {
                println!("{:?}", cc);
                complete_commands = rest.as_mut();
            },
            ast::CompleteCommands::Nil => break,
        }
    }
}
