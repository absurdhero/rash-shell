use ast;

pub fn eval(program: &ast::Program) -> () {
    for complete_command in &program.commands.complete_commands {
        println!("{:?}", complete_command);
    }
}
