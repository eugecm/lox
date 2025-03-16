use std::collections::HashMap;

use crate::syntax::{Identifier, Literal};

#[derive(Debug)]
pub struct Environment {
    stack: Vec<HashMap<Identifier, Literal>>,
}

impl<'a> Default for Environment {
    fn default() -> Self {
        Self {
            stack: vec![HashMap::default()],
        }
    }
}

impl Environment {
    pub fn push_env(&mut self) {
        self.stack.push(HashMap::default());
    }

    pub fn pop_env(&mut self) {
        self.stack.pop();
        assert!(
            !self.stack.is_empty(),
            "last environment was popped, but that's impossible"
        )
    }

    pub fn define(&mut self, name: Identifier, value: Literal) {
        self.stack
            .last_mut()
            .expect("must have at least 1 environment")
            .insert(name, value);
    }

    pub fn get(&self, name: &Identifier) -> Option<Literal> {
        self.stack.iter().rev().find_map(|h| h.get(name).cloned())
    }

    pub fn mutate(&mut self, name: &Identifier, value: Literal) -> Option<Literal> {
        let mut name = name.clone();
        for env in self.stack.iter_mut().rev() {
            match env.entry(name) {
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
}
