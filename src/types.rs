use std::{cell::RefCell, collections::HashMap, fmt::Display, rc::Rc};

use crate::interpreter::Interpreter;

#[derive(Clone)]
pub enum Object {
    String(Rc<str>),
    Number(f64),
    Boolean(bool),
    Callable(Rc<dyn Callable>),
    Null, // eww
}

impl std::fmt::Debug for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Object::String(s) => write!(f, "{s:?}"),
            Object::Number(n) => write!(f, "{n:?}"),
            Object::Boolean(v) => write!(f, "{v:?}"),
            Object::Callable(c) => write!(f, "<callable:{}>", c.arity()),
            Object::Null => write!(f, "null"),
        }
    }
}

macro_rules! literal_or_false {
    ( $s:ident, $left:ident ) => {
        match $s {
            Object::$left(left) => left,
            _ => return false,
        }
    };
}
impl PartialEq for Object {
    fn eq(&self, other: &Self) -> bool {
        match other {
            Object::String(right) => {
                let left = literal_or_false!(self, String);
                left == right
            }
            Object::Number(right) => {
                let left = literal_or_false!(self, Number);
                left == right
            }
            Object::Boolean(right) => {
                let left = literal_or_false!(self, Boolean);
                left == right
            }
            Object::Callable(_right) => {
                unimplemented!("can't compare functions yet");
            }
            Object::Null => matches!(self, Object::Null),
        }
    }
}

impl Object {
    pub fn is_truthy(&self) -> bool {
        match self {
            Object::Boolean(value) => *value,
            typ => panic!("invalid non-boolean value {typ:?} evaluated to truthy"),
        }
    }
}

impl Display for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Object::String(s) => write!(f, "{s}")?,
            Object::Number(n) => write!(f, "{n}")?,
            Object::Boolean(v) => write!(f, "{v}")?,
            Object::Callable(c) => write!(f, "<callable:{}>", c.arity())?,
            Object::Null => write!(f, "null")?,
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Identifier(pub Rc<str>);

impl Display for Identifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0, f)
    }
}

pub trait Callable {
    fn arity(&self) -> usize;
    fn call(&self, interpreter: &mut Interpreter, args: &[Object]) -> Object;
}

pub type Scope = Rc<RefCell<HashMap<Identifier, Object>>>;
