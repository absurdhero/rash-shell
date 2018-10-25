use ast;

pub fn eval(program: &ast::Program) -> () {
    for cc in &program.commands.complete_commands {
        complete_command(cc);
    }
}

fn complete_command(cc: &ast::CompleteCommand) {
    for (op, list) in &cc.and_ors {
        andor_list(*op == ast::TermOp::Amp, list);
    }
}

fn andor_list(async: bool, list: &ast::AndOr) {
    for (op, pipeline) in &list.pipelines {
        let result = exec_pipeline(pipeline);
        match op {
            ast::AndOrOp::And => {
                if result {
                    continue;
                }
            },
            ast::AndOrOp::Or => {
                if result {
                    break;
                }
            },
        }
    }
}

fn exec_pipeline(pipeline: &ast::Pipeline) -> bool {
    for command in &pipeline.commands {
        println!("{:?}", command);
    }
    return true;
}