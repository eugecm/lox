use crate::{
    environment::Environment,
    eval::eval,
    syntax::{Declaration, Identifier, Program, Stmt},
};

#[derive(Debug)]
pub struct Interpreter {
    environment: Environment,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            environment: Environment::default(),
        }
    }

    pub fn interpret(&mut self, prog: Program) {
        match prog {
            Program::Declarations(decls) => {
                for decl in decls {
                    self.execute(&decl);
                }
            }
        }
    }

    fn execute_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::ExprStmt(expr) => {
                let _ = eval(expr, &mut self.environment);
            }
            Stmt::IfStmt {
                condition,
                then_branch,
                else_branch,
            } => {
                let condition_value = match eval(condition, &mut self.environment) {
                    crate::syntax::Literal::Boolean(value) => value,
                    literal => panic!("if condition can only be boolean, got '{literal:?}'"),
                };
                if condition_value {
                    self.execute_stmt(then_branch);
                } else if let Some(else_branch) = else_branch
                .as_ref() { self.execute_stmt(else_branch) }
            }
            Stmt::PrintStmt(expr) => {
                let value = eval(expr, &mut self.environment);
                println!("{value}")
            }
            Stmt::WhileStmt { condition, body } => loop {
                let condition_value = match eval(condition, &mut self.environment) {
                    crate::syntax::Literal::Boolean(value) => value,
                    literal => panic!("while condition can only be boolean, got '{literal:?}'"),
                };
                if condition_value {
                    self.execute_stmt(body);
                } else {
                    break;
                }
            },
            Stmt::Block(decls) => {
                self.execute_block(decls);
            }
        }
    }

    pub fn execute(&mut self, decl: &Declaration) {
        match decl {
            Declaration::Statement(stmt) => {
                self.execute_stmt(stmt);
            }
            Declaration::Var {
                identifier,
                expression,
            } => {
                let value = eval(expression, &mut self.environment);
                self.environment
                    .define(Identifier(identifier.clone()), value);
            }
        }
    }

    fn execute_block(&mut self, statements: &[Declaration]) {
        self.environment.push_env();

        for stmt in statements {
            self.execute(stmt);
        }

        self.environment.pop_env();
    }
}
