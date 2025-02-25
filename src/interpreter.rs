use crate::{
    eval::eval,
    syntax::{Declaration, Program, Stmt},
};

pub struct Interpreter {}

impl<'b: 'a, 'a> Interpreter {
    pub fn new() -> Self {
        Self {}
    }

    pub fn interpret(&'a mut self, prog: Program<'b>) {
        match prog {
            Program::Declarations(decls) => {
                for decl in decls {
                    self.execute(decl);
                }
            }
        }
    }

    pub fn execute(&'a mut self, decl: Declaration<'b>) {
        match decl {
            Declaration::Statement(Stmt::ExprStmt(expr)) => {
                let _ = eval(&expr);
            }
            Declaration::Statement(Stmt::PrintStmt(expr)) => {
                let value = eval(&expr);
                println!("{value}")
            }
            Declaration::Var {
                identifier,
                expression,
            } => todo!(),
        }
    }
}
