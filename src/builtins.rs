use std::{
    rc::Rc,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::types::{Callable, Environment, Object};

pub fn get_builtins() -> Vec<(&'static str, Object)> {
    vec![("clock", clock_fn())]
}

fn clock_fn() -> Object {
    Object::Callable(Rc::new(ClockFn {}))
}

struct ClockFn;
impl Callable for ClockFn {
    fn arity(&self) -> usize {
        0
    }

    fn call(&self, _: &Box<dyn Environment>, _: &[crate::types::Object]) -> Object {
        let a = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        Object::Number(a.as_secs_f64())
    }
}
