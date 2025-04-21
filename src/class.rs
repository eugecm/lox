use std::{cell::RefCell, collections::HashMap, rc::Rc};

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

    fn bind(&self, _instance: &ClassInstance) -> Object {
        unimplemented!("can't call bind on a class")
    }
}

pub type ClassInstanceState = Rc<RefCell<HashMap<Identifier, Object>>>;

#[derive(Clone)]
pub struct ClassInstance {
    pub(crate) class: Class,
    pub(crate) fields: ClassInstanceState,
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

        if let Some(Object::Callable(method)) = self.class.find_method(name) {
            return method.bind(&self);
        }

        panic!("Undefined property '{name}'")
    }

    pub fn set(&self, name: Identifier, value: Object) {
        self.fields.borrow_mut().insert(name, value);
    }
}
