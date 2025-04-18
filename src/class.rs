use std::{cell::RefCell, collections::HashMap};

use crate::{
    callable::FunctionRef,
    types::{Callable, Identifier, Object},
};

type Methods = HashMap<Identifier, FunctionRef>;

#[derive(Debug, Clone)]
pub struct Class {
    pub(crate) name: Identifier,
    pub(crate) methods: Methods,
}

impl Class {
    pub fn new(name: Identifier, methods: Methods) -> Self {
        Self { name, methods }
    }

    fn find_method(&self, name: &Identifier) -> Option<Object> {
        self.methods.get(name).cloned().map(|o| Object::Callable(o))
    }
}

impl Callable for Class {
    fn arity(&self) -> usize {
        0
    }

    fn call(
        &self,
        _interpreter: &mut crate::interpreter::Interpreter,
        _args: &[crate::types::Object],
    ) -> crate::types::Object {
        crate::types::Object::ClassInstance(ClassInstance::new(self.clone()).into())
    }
}

pub struct ClassInstance {
    pub(crate) class: Class,
    fields: RefCell<HashMap<Identifier, Object>>,
}

impl ClassInstance {
    pub fn new(class: Class) -> Self {
        Self {
            class,
            fields: Default::default(),
        }
    }

    pub fn get(&self, name: &Identifier) -> Object {
        if let Some(field) = self.fields.borrow().get(name).cloned() {
            return field;
        }

        if let Some(method) = self.class.find_method(name) {
            return method;
        }

        panic!("Undefined property '{name}'")
    }

    pub fn set(&self, name: Identifier, value: Object) {
        self.fields.borrow_mut().insert(name, value);
    }
}
