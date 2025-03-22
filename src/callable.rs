use crate::{
    interpreter::Interpreter,
    syntax::FunctionStmt,
    types::{Callable, Identifier, Object, Scope},
};

pub struct Function {
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

    fn call(&self, interpreter: &mut Interpreter, args: &[Object]) -> Object {
        let scope = Scope::default();
        for (i, param) in self.decl.parameters.iter().enumerate() {
            // Should probably create a 'define' in scope
            scope
                .borrow_mut()
                .insert(Identifier(param.lexeme.clone()), args[i].clone());
        }

        // The "catch" statement
        match interpreter.execute_block(&self.decl.body, scope) {
            Ok(x) | Err(x) => x,
        }
    }
}
