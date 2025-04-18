use std::{cell::RefCell, collections::HashMap};

use crate::types::{Callable, Identifier, Object};

#[derive(Debug, Clone)]
pub struct Class {
    pub(crate) name: Identifier,
}

impl Class {
    pub fn new(name: Identifier) -> Self {
        Self { name }
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
        self.fields
            .borrow()
            .get(name)
            .cloned()
            .unwrap_or_else(|| panic!("undefined property '{name}'"))
    }

    pub fn set(&self, name: Identifier, value: Object) {
        self.fields.borrow_mut().insert(name, value);
    }
}
