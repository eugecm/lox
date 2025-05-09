use std::{collections::HashMap, rc::Rc};

use crate::{
    builtins::get_builtins,
    callable::Function,
    class::Class,
    environment::{EnvRef, Environment},
    scanner::{Token, TokenType},
    syntax::{Declaration, Expr, ExprKind, Program, Stmt},
    types::{Identifier, Object},
};

type Flow<T> = Result<T, T>;

#[derive(Debug)]
pub struct Interpreter {
    environment: EnvRef,
    globals: EnvRef,
    locals: HashMap<u64, usize>,
}

impl Interpreter {
    pub fn new() -> Self {
        // Initialize globals
        let globals = EnvRef::default();
        for (name, builtin) in get_builtins() {
            globals
                .borrow_mut()
                .define(Identifier(name.into()), builtin);
        }

        let environment = globals.clone();

        Self {
            globals,
            environment,
            locals: HashMap::default(),
        }
    }

    pub fn interpret(&mut self, prog: Program) {
        match prog {
            Program::Declarations(decls) => {
                for decl in decls {
                    let _ = self.execute(&decl);
                }
            }
        }
    }

    pub fn execute_stmt(&mut self, stmt: &Stmt) -> Flow<Object> {
        match stmt {
            Stmt::Expr(expr) => Flow::Ok(self.eval(expr)),
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let condition_value = match self.eval(condition) {
                    crate::types::Object::Boolean(value) => value,
                    literal => panic!("if condition can only be boolean, got '{literal:?}'"),
                };
                if condition_value {
                    Flow::Ok(self.execute_stmt(then_branch)?)
                } else if let Some(else_branch) = else_branch.as_ref() {
                    Flow::Ok(self.execute_stmt(else_branch)?)
                } else {
                    Flow::Ok(Object::Null)
                }
            }
            Stmt::Print(expr) => {
                let value = self.eval(expr);
                println!("{value}");
                Flow::Ok(Object::Null)
            }
            Stmt::While { condition, body } => loop {
                let condition_value = match self.eval(condition) {
                    crate::types::Object::Boolean(value) => value,
                    literal => panic!("while condition can only be boolean, got '{literal:?}'"),
                };
                if condition_value {
                    self.execute_stmt(body)?
                } else {
                    return Flow::Ok(Object::Null);
                };
            },
            Stmt::Block(decls) => Flow::Ok(
                self.execute_block(decls, Environment::new_ref(Some(self.environment.clone())))?,
            ),
            Stmt::FunctionDecl(function_stmt) => {
                let fun = Object::Callable(Rc::new(Function::new(
                    function_stmt.clone(),
                    self.environment.clone(),
                    false,
                )));
                self.environment
                    .borrow_mut()
                    .define(function_stmt.identifier.clone(), fun);
                Flow::Ok(Object::Null)
            }
            Stmt::Return { value } => Flow::Err(self.eval(value)),
            Stmt::ClassDecl(class_decl) => {
                let superclass = class_decl.superclass.as_ref().map(|superclass| {
                    let superclass = self.eval(superclass);
                    let Object::Class(superclass) = superclass else {
                        panic!("superclass is not a class!");
                    };
                    superclass
                });

                self.environment
                    .borrow_mut()
                    .define(class_decl.name.clone(), Object::Null);

                if let Some(superclass) = &superclass {
                    self.environment = Environment::new_ref(Some(self.environment.clone()));
                    self.environment
                        .borrow_mut()
                        .define("super".into(), Object::Class(superclass.clone()));
                }

                let mut methods = HashMap::new();
                for method in &class_decl.methods {
                    let function = Rc::new(Function::new(
                        method.clone(),
                        self.environment.clone(),
                        method.identifier.as_ref() == "init",
                    ));
                    methods.insert(method.identifier.clone(), function);
                }
                let class = Class::new(class_decl.name.clone(), superclass.clone(), methods);

                if superclass.is_some() {
                    let parent = self.environment.borrow().parent.clone().unwrap();
                    self.environment = parent;
                }

                self.environment
                    .borrow_mut()
                    .mutate(&class_decl.name, Object::Class(class.into()));
                Flow::Ok(Object::Null)
            }
        }
    }

    pub fn execute(&mut self, decl: &Declaration) -> Flow<Object> {
        match decl {
            Declaration::Statement(stmt) => Flow::Ok(self.execute_stmt(stmt)?),
            Declaration::Var {
                identifier,
                expression,
            } => {
                let value = self.eval(expression);
                self.environment
                    .borrow_mut()
                    .define(identifier.clone(), value);
                Flow::Ok(Object::Null)
            }
        }
    }

    pub fn execute_block(&mut self, statements: &[Declaration], env: EnvRef) -> Flow<Object> {
        let prev_env = self.environment.clone();
        self.environment = env;

        let mut last = Object::Null;
        for stmt in statements {
            last = match self.execute(stmt) {
                Ok(v) => v,
                Err(v) => {
                    self.environment = prev_env;
                    return Flow::Err(v);
                }
            };
        }

        self.environment = prev_env;
        Flow::Ok(last)
    }

    pub fn eval(&mut self, expr: &Expr) -> Object {
        let expr_kind = &expr.kind;
        match expr_kind {
            ExprKind::Binary { left, op, right } => self.eval_binary(left, op, right),
            ExprKind::Grouping { expr } => self.eval(expr),
            ExprKind::Literal { value } => self.eval_literal(value),
            ExprKind::Unary { op, right } => self.eval_unary(op, right),
            ExprKind::Var { name } => self.eval_var(name.clone(), expr),
            ExprKind::Assign { name, expr } => self.eval_assign(name, expr),
            ExprKind::Logical { left, op, right } => self.eval_logical(left, op, right),
            ExprKind::Call {
                callee,
                parens: _,
                args,
            } => self.eval_call(callee, args),
            ExprKind::Get { name, object } => self.eval_get(name, object),
            ExprKind::Set {
                object,
                name,
                value,
            } => self.eval_set(object, name, value),
            ExprKind::This { token } => self.lookup_var(token.clone(), expr),
            ExprKind::Super { token: _, method } => {
                let distance = *self.locals.get(&expr.id).unwrap();
                let Object::Class(superclass) =
                    self.environment.borrow().get_at(distance, &"super".into())
                else {
                    panic!("bug: environment.get(super) did not return a class")
                };
                let Object::ClassInstance(object) = self
                    .environment
                    .borrow()
                    .get_at(distance - 1, &"this".into())
                else {
                    panic!("bug: environment.get(this) did not return a class instance");
                };

                let method = superclass.find_method(method);
                let Some(Object::Callable(method)) = method else {
                    panic!("method not found {method:?}");
                };
                method.bind(&object)
            }
        }
    }

    fn eval_call(&mut self, callee: &Expr, args: &[Expr]) -> Object {
        let callee = self.eval(callee);

        let arguments: Vec<_> = args.iter().map(|arg| self.eval(arg)).collect();

        let callable = match callee {
            Object::Callable(c) => c,
            Object::Class(c) => c,
            Object::String(_)
            | Object::Number(_)
            | Object::Boolean(_)
            | Object::ClassInstance(_)
            | Object::Null => panic!("'{callee}' is not callable"),
        };

        if callable.arity() != arguments.len() {
            let arity = callable.arity();
            let n_args = arguments.len();
            panic!("called fn/{arity} with {n_args}");
        }
        callable.call(self, &arguments)
    }

    fn eval_logical(&mut self, left: &Expr, op: &Token, right: &Expr) -> Object {
        let left = self.eval(left);

        if op.typ == TokenType::Or {
            if left.is_truthy() {
                return left;
            }
        } else if !left.is_truthy() {
            return left;
        }

        self.eval(right)
    }

    fn eval_literal(&mut self, value: &Object) -> Object {
        value.clone()
    }

    fn eval_unary(&mut self, op: &Token, right: &Expr) -> Object {
        match op.typ {
            TokenType::Minus => {
                let sub = self.eval(right);
                match sub {
                    Object::Number(n) => Object::Number(-n),
                    _ => panic!("invalid "),
                }
            }
            t => {
                panic!("unexpected token {t:?}. Expecting '-'")
            }
        }
    }

    fn eval_binary(&mut self, left: &Expr, op: &Token, right: &Expr) -> Object {
        let left = self.eval(left);
        let right = self.eval(right);
        match (left, op.typ, right) {
            // Numbers
            (Object::Number(left), TokenType::Minus, Object::Number(right)) => {
                Object::Number(left - right)
            }
            (Object::Number(left), TokenType::Plus, Object::Number(right)) => {
                Object::Number(left + right)
            }
            (Object::Number(left), TokenType::Slash, Object::Number(right)) => {
                Object::Number(left / right)
            }
            (Object::Number(left), TokenType::Star, Object::Number(right)) => {
                Object::Number(left * right)
            }
            (Object::Number(left), TokenType::Greater, Object::Number(right)) => {
                Object::Boolean(left > right)
            }
            (Object::Number(left), TokenType::GreaterEqual, Object::Number(right)) => {
                Object::Boolean(left >= right)
            }
            (Object::Number(left), TokenType::Less, Object::Number(right)) => {
                Object::Boolean(left < right)
            }
            (Object::Number(left), TokenType::LessEqual, Object::Number(right)) => {
                Object::Boolean(left <= right)
            }
            (left, TokenType::EqualEqual, right) => Object::Boolean(is_equal(left, right)),
            (left, TokenType::BangEqual, right) => Object::Boolean(!is_equal(left, right)),

            (Object::String(left), TokenType::Plus, Object::String(right)) => {
                Object::String(format!("{left}{right}").into())
            }

            (left, op, right) => {
                panic!(
                "invalid operator '{op:?}' for '{left:?}' and '{right:?}'. This isn't javascript"
            )
            }
        }
    }

    fn eval_var(&mut self, name: Identifier, expr: &Expr) -> Object {
        self.lookup_var(name, expr)
    }

    pub fn resolve(&mut self, expr: &Expr, depth: usize) {
        self.locals.insert(expr.id, depth);
    }

    fn lookup_var(&self, name: Identifier, expr: &Expr) -> Object {
        if let Some(distance) = self.locals.get(&expr.id) {
            return self.environment.borrow().get_at(*distance, &name);
        } else {
            return self.globals.borrow().get(&name).unwrap_or_else(|| {
                panic!("could not find variable {name:?} in environment nor global scope. locals={:?}, environment={:?}, global={:?}", self.locals, self.environment, self.globals)
            });
        }
    }

    fn eval_assign(&mut self, name: &Identifier, expr: &Expr) -> Object {
        let value = self.eval(expr);
        let distance = self.locals.get(&expr.id);
        if let Some(distance) = distance {
            self.environment
                .borrow()
                .assign_at(*distance, name.clone(), value.clone());
        } else {
            self.environment.borrow().mutate(name, value.clone());
        }

        value
    }

    fn eval_get(&mut self, name: &Identifier, object: &Expr) -> Object {
        let obj = self.eval(object);
        let Object::ClassInstance(ins) = obj else {
            panic!("only instances have properties");
        };

        ins.get(name)
    }

    fn eval_set(&mut self, object: &Expr, name: &Identifier, value: &Expr) -> Object {
        let obj = self.eval(object);
        let Object::ClassInstance(ins) = obj else {
            panic!("only instances have fields");
        };
        let value = self.eval(value);
        ins.set(name.clone(), value.clone());
        value
    }
}

fn is_equal(left: Object, right: Object) -> bool {
    match (left, right) {
        (Object::String(left), Object::String(right)) => left == right,
        (Object::Number(left), Object::Number(right)) => left == right,
        (Object::Boolean(left), Object::Boolean(right)) => left == right,
        (Object::Null, Object::Null) => true,
        _ => false,
    }
}
