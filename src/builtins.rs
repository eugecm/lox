use std::{
    rc::Rc,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{
    interpreter::Interpreter,
    types::{Callable, Object},
};

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

    fn call(&self, _: &mut Interpreter, _: &[crate::types::Object]) -> Object {
        let a = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        Object::Number(a.as_secs_f64())
    }

    fn bind(&self, _instance: &crate::class::ClassInstance) -> Object {
        unimplemented!("can't bind clock")
    }
}
