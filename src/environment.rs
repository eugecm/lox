use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::types::{Identifier, Object, Scope};

#[derive(Debug)]
pub struct Environment {
    parent: Option<Rc<RefCell<Environment>>>,
    values: HashMap<Identifier, Object>,
}

impl Environment {
    pub fn define(&self, name: Identifier, value: Object) {
        self.stack
            .borrow_mut()
            .last_mut()
            .expect("must have at least 1 environment")
            .borrow_mut()
            .insert(name, value);
    }

    pub fn get(&self, name: &Identifier) -> Option<Object> {
        self.stack
            .borrow()
            .iter()
            .rev()
            .find_map(|h| h.borrow().get(name).cloned())
    }

    pub fn mutate(&self, name: &Identifier, value: Object) -> Option<Object> {
        let mut name = name.clone();
        for env in self.stack.borrow_mut().iter_mut().rev() {
            match env.borrow_mut().entry(name) {
                std::collections::hash_map::Entry::Occupied(mut entry) => {
                    return Some(entry.insert(value))
                }
                std::collections::hash_map::Entry::Vacant(entry) => {
                    // Return ownership of key back to name
                    name = entry.into_key();
                }
            }
        }
        None
    }

    pub fn push(&self, scope: Scope) {
        self.stack.borrow_mut().push(scope);
    }

    pub fn pop(&self) {
        self.stack.borrow_mut().pop();
        assert!(
            !self.stack.borrow().is_empty(),
            "last environment was popped, but that's impossible"
        )
    }
}

impl Default for Environment {
    fn default() -> Self {
        Self {
            stack: RefCell::new(vec![Scope::default()]),
        }
    }
}
