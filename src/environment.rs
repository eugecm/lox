use std::collections::HashMap;

use crate::syntax::Literal;

#[derive(Debug)]
pub struct Environment<'a> {
    stack: Vec<HashMap<&'a str, Literal<'a>>>,
}

impl<'a> Default for Environment<'a> {
    fn default() -> Self {
        Self {
            stack: vec![HashMap::default()],
        }
    }
}

impl<'a, 's> Environment<'a> {
    pub fn push_env(&'s mut self) {
        self.stack.push(HashMap::default());
    }

    pub fn pop_env(&'s mut self) {
        self.stack.pop();
        assert!(
            self.stack.len() > 0,
            "last environment was popped, but that's impossible"
        )
    }

    pub fn define(&mut self, name: &'a str, value: Literal<'a>) {
        self.stack
            .last_mut()
            .expect("must have at least 1 environment")
            .insert(name, value);
    }

    pub fn get(&self, name: &'a str) -> Option<Literal<'a>> {
        self.stack.iter().rev().find_map(|h| h.get(name).cloned())
    }

    pub fn mutate(&mut self, name: &'a str, value: Literal<'a>) -> Option<Literal<'a>> {
        self.stack
            .iter_mut()
            .rev()
            .find_map(|h| match h.entry(name) {
                std::collections::hash_map::Entry::Occupied(entry) => Some(entry),
                std::collections::hash_map::Entry::Vacant(_) => None,
            })
            .map(|mut entry| entry.insert(value))
    }
}
