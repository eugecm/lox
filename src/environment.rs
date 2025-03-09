use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::syntax::Literal;

#[derive(Debug, Default)]
pub struct Environment<'a> {
    enclosing: Option<Rc<Environment<'a>>>,

    values: RefCell<HashMap<&'a str, Literal<'a>>>,
}

impl<'a> Environment<'a> {
    pub fn new(enclosing: Rc<Environment<'a>>) -> Self {
        Self {
            enclosing: Some(enclosing),
            values: RefCell::new(HashMap::default()),
        }
    }

    pub fn define(&self, name: &'a str, value: Literal<'a>) {
        self.values.borrow_mut().insert(name, value);
    }

    pub fn get(&self, name: &'a str) -> Option<Literal<'a>> {
        if let Some(v) = self.values.borrow().get(name).cloned() {
            return Some(v);
        }

        self.enclosing.as_ref().and_then(|e| e.get(name))
    }

    pub fn mutate(&self, name: &'a str, value: Literal<'a>) -> Option<Literal<'a>> {
        let mut values = self.values.borrow_mut();
        if let Some(previous) = values.get_mut(name) {
            *previous = value;
            return Some(previous.clone());
        }

        self.enclosing.as_ref().and_then(|e| e.mutate(name, value))
    }
}
