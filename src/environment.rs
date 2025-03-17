use std::cell::RefCell;

use crate::types::{Environment, Identifier, Object, Scope};

#[derive(Debug)]
pub struct GlobalScope {
    stack: RefCell<Vec<Scope>>,
}

impl Default for GlobalScope {
    fn default() -> Self {
        Self {
            stack: RefCell::new(vec![Scope::default()]),
        }
    }
}

impl Environment for GlobalScope {
    fn define(&self, name: Identifier, value: Object) {
        self.stack
            .borrow_mut()
            .last_mut()
            .expect("must have at least 1 environment")
            .borrow_mut()
            .insert(name, value);
    }

    fn get(&self, name: &Identifier) -> Option<Object> {
        self.stack
            .borrow()
            .iter()
            .rev()
            .find_map(|h| h.borrow().get(name).cloned())
    }

    fn mutate(&self, name: &Identifier, value: Object) -> Option<Object> {
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

    fn push(&self) {
        self.stack.borrow_mut().push(Scope::default());
    }

    fn pop(&self) {
        self.stack.borrow_mut().pop();
        assert!(
            !self.stack.borrow().is_empty(),
            "last environment was popped, but that's impossible"
        )
    }

    fn globals(&self) -> Scope {
        let stack = self.stack.borrow();
        stack[0].clone()
    }
}
