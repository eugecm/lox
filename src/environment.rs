use std::{
    cell::RefCell,
    collections::{hash_map::Entry, HashMap},
    fmt::Debug,
    rc::Rc,
};

use crate::types::{Identifier, Object};

pub type EnvRef = Rc<RefCell<Environment>>;
pub type Values = Rc<RefCell<HashMap<Identifier, Object>>>;

pub struct Environment {
    pub parent: Option<EnvRef>,
    values: Values,
}

impl Debug for Environment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.values.borrow(), f)?;
        write!(f, "->")?;
        Debug::fmt(&self.parent, f)
    }
}

impl Environment {
    pub fn new_ref(parent: Option<EnvRef>) -> EnvRef {
        Rc::new(RefCell::new(Self {
            parent,
            values: Default::default(),
        }))
    }

    pub fn define(&self, name: Identifier, value: Object) {
        self.values.borrow_mut().insert(name, value);
    }

    pub fn get(&self, name: &Identifier) -> Option<Object> {
        let value = self.values.borrow().get(name).cloned();
        if value.is_none() {
            return self.parent.as_ref().and_then(|p| p.borrow().get(name));
        }

        value
    }

    pub fn assign_at(&self, distance: usize, name: Identifier, value: Object) {
        let ancestor: Values = self.ancestor(distance);
        ancestor.borrow_mut().insert(name, value);
    }

    pub fn get_at(&self, distance: usize, name: &Identifier) -> Object {
        let ancestor: Values = self.ancestor(distance);
        let x = ancestor.borrow().get(name).cloned().unwrap_or_else(|| {
            panic!("could not find {name:?} in scope {ancestor:?} for distance {distance}")
        });
        x
    }

    fn ancestor(&self, distance: usize) -> Values {
        if distance == 0 {
            return self.values.clone();
        }
        self.parent
            .as_ref()
            .map(|parent| parent.borrow().ancestor(distance - 1).clone())
            .unwrap()
    }

    pub fn mutate(&self, name: &Identifier, value: Object) -> Option<Object> {
        let name = name.clone();
        match self.values.borrow_mut().entry(name.clone()) {
            Entry::Occupied(mut entry) => Some(entry.insert(value)),
            Entry::Vacant(_) => {
                return self
                    .parent
                    .as_ref()
                    .and_then(|p| p.borrow_mut().mutate(&name, value))
            }
        }
    }
}

impl Default for Environment {
    fn default() -> Self {
        Self {
            parent: None,
            values: Default::default(),
        }
    }
}
