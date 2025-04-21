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
    is_initializer: bool,
}

impl Function {
    pub fn new(decl: FunctionStmt, closure: EnvRef, is_initializer: bool) -> Self {
        Self {
            decl,
            closure,
            is_initializer,
        }
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
        let ret_value = match interpreter.execute_block(&self.decl.body, env) {
            Ok(x) => x,
            Err(x) => return x,
        };

        if self.is_initializer {
            self.closure.borrow().get_at(0, &"this".into())
        } else {
            ret_value
        }
    }

    fn bind(&self, instance: &crate::class::ClassInstance) -> Object {
        let env = Environment::new_ref(Some(self.closure.clone()));
        env.borrow_mut().define(
            Identifier("this".into()),
            Object::ClassInstance(instance.clone().into()),
        );
        Object::Callable(Rc::new(Function::new(
            self.decl.clone(),
            env,
            self.is_initializer,
        )))
    }
}
