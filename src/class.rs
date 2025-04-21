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
    pub(crate) superclass: Option<Rc<Class>>,
}

impl Class {
    pub fn new(name: Identifier, superclass: Option<Rc<Class>>, methods: Methods) -> Self {
        Self {
            name,
            superclass,
            methods,
        }
    }

    fn find_method(&self, name: &Identifier) -> Option<Object> {
        self.methods
            .get(name)
            .cloned()
            .map(|o| Object::Callable(o))
            .or_else(|| self.superclass.as_ref().and_then(|s| s.find_method(name)))
    }
}

impl Callable for Class {
    fn arity(&self) -> usize {
        let initializer = self.find_method(&"init".into());
        match initializer {
            Some(Object::Callable(t)) => t.arity(),
            None => 0,
            Some(e) => panic!("init method must be a callable, got {e} instead"),
        }
    }

    fn call(
        &self,
        interpreter: &mut crate::interpreter::Interpreter,
        args: &[crate::types::Object],
    ) -> crate::types::Object {
        let instance = ClassInstance::new(self.clone());

        let initializer = self.find_method(&"init".into());
        if let Some(initializer) = initializer {
            let Object::Callable(initializer) = initializer else {
                panic!("initializer must be a callable function");
            };

            let Object::Callable(initializer) = initializer.bind(&instance) else {
                panic!("initializer->bind did not return a callable, this is a bug");
            };

            initializer.call(interpreter, args);
        }

        Object::ClassInstance(instance.into())
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
