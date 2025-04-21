use std::rc::Rc;

use crate::{
    environment::{EnvRef, Environment},
    interpreter::Interpreter,
    syntax::FunctionStmt,
    types::{Callable, Identifier, Object},
};

pub type FunctionRef = Rc<Function>;

#[derive(Debug, Clone)]
pub struct Function {
    decl: FunctionStmt,
    closure: EnvRef,
}

impl Function {
    pub fn new(decl: FunctionStmt, closure: EnvRef) -> Self {
        Self { decl, closure }
    }
}

impl Callable for Function {
    fn arity(&self) -> usize {
        self.decl.parameters.len()
    }

    fn call(&self, interpreter: &mut Interpreter, args: &[Object]) -> Object {
        let env = Environment::new_ref(Some(self.closure.clone()));
        for (i, param) in self.decl.parameters.iter().enumerate() {
            env.borrow_mut()
                .define(Identifier(param.lexeme.clone()), args[i].clone());
        }

        // The "catch" statement
        match interpreter.execute_block(&self.decl.body, env) {
            Ok(x) | Err(x) => x,
        }
    }

    fn bind(&self, instance: &crate::class::ClassInstance) -> Object {
        let env = Environment::new_ref(Some(self.closure.clone()));
        env.borrow_mut().define(
            Identifier("this".into()),
            Object::ClassInstance(instance.clone().into()),
        );
        Object::Callable(Rc::new(Function::new(self.decl.clone(), env)))
    }
}
