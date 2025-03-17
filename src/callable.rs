use crate::{
    syntax::FunctionStmt,
    types::{Callable, Environment, Identifier, Object},
};

struct Function {
    decl: FunctionStmt,
}

impl Function {
    pub fn new(decl: FunctionStmt) -> Self {
        Self { decl }
    }
}

impl Callable for Function {
    fn arity(&self) -> usize {
        self.decl.parameters.len()
    }

    fn call(&self, env: &Box<dyn Environment>, args: &[Object]) -> Object {
        for (i, param) in self.decl.parameters.iter().enumerate() {
            env.define(Identifier(param.lexeme), args[i]);
        }
    }
}
