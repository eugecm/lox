use std::rc::Rc;

use crate::{
    environment::Environment,
    eval::eval,
    syntax::{Declaration, Program, Stmt},
};

#[derive(Debug)]
pub struct Interpreter<'a> {
    environment: Rc<Environment<'a>>,
}

impl<'sc: 's, 's> Interpreter<'sc> {
    pub fn new() -> Self {
        Self {
            environment: Rc::new(Environment::default()),
        }
    }

    pub fn interpret(&'s mut self, prog: Program<'sc>) {
        match prog {
            Program::Declarations(decls) => {
                for decl in decls {
                    self.execute(decl);
                }
            }
        }
    }

    pub fn execute(&'s mut self, decl: Declaration<'sc>) {
        match decl {
            Declaration::Statement(Stmt::ExprStmt(expr)) => {
                let _ = eval(&expr, &mut self.environment);
            }
            Declaration::Statement(Stmt::PrintStmt(expr)) => {
                let value = eval(&expr, &mut self.environment);
                println!("{value}")
            }
            Declaration::Statement(Stmt::Block(decls)) => {
                self.execute_block(decls, Rc::new(Environment::new(self.environment.clone())));
            }
            Declaration::Var {
                identifier,
                expression,
            } => {
                let value = eval(&expression, &mut self.environment);
                self.environment.define(identifier, value);
            }
        }
    }

    fn execute_block(&'s mut self, statements: Vec<Declaration<'sc>>, env: Rc<Environment<'sc>>) {
        let previous = self.environment.clone();
        self.environment = env;

        for stmt in statements {
            self.execute(stmt);
        }

        self.environment = previous;
    }
}
