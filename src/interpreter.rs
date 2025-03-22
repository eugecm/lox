use std::rc::Rc;

use crate::{
    builtins::get_builtins,
    callable::Function,
    environment::GlobalScope,
    scanner::{Token, TokenType},
    syntax::{Declaration, Expr, Program, Stmt},
    types::{Environment, Identifier, Object, Scope},
};

type Flow<T> = Result<T, T>;

#[derive(Debug)]
pub struct Interpreter {
    environment: GlobalScope,
}

impl Interpreter {
    pub fn new() -> Self {
        // Initialize globals
        let environment = GlobalScope::default();
        for (name, builtin) in get_builtins() {
            environment.define(Identifier(name.into()), builtin);
        }

        Self { environment }
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

    pub fn globals(&self) -> crate::types::Scope {
        self.environment.globals()
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
            Stmt::Block(decls) => Flow::Ok(self.execute_block(decls, Scope::default())?),
            Stmt::Function(function_stmt) => {
                let fun = Object::Callable(Rc::new(Function::new(function_stmt.clone())));
                self.environment
                    .define(function_stmt.identifier.clone(), fun);
                Flow::Ok(Object::Null)
            }
            Stmt::Return { value } => Flow::Err(self.eval(value)),
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
                self.environment.define(identifier.clone(), value);
                Flow::Ok(Object::Null)
            }
        }
    }

    pub fn execute_block(&mut self, statements: &[Declaration], scope: Scope) -> Flow<Object> {
        self.environment.push(scope);

        let mut last = Object::Null;
        for stmt in statements {
            last = match self.execute(stmt) {
                Ok(v) => v,
                Err(v) => {
                    self.environment.pop();
                    return Flow::Err(v);
                }
            };
        }

        self.environment.pop();
        Flow::Ok(last)
    }

    pub fn eval(&mut self, expr: &Expr) -> Object {
        match expr {
            Expr::Binary { left, op, right } => self.eval_binary(left, op, right),
            Expr::Grouping { expr } => self.eval(expr),
            Expr::Literal { value } => self.eval_literal(value),
            Expr::Unary { op, right } => self.eval_unary(op, right),
            Expr::Var { name } => self.eval_var(name),
            Expr::Assign { name, expr } => self.eval_assign(name, expr),
            Expr::Logical { left, op, right } => self.eval_logical(left, op, right),
            Expr::Call {
                callee,
                parens: _,
                args,
            } => self.eval_call(callee, args),
        }
    }

    fn eval_call(&mut self, callee: &Expr, args: &[Expr]) -> Object {
        let callee = self.eval(callee);

        let arguments: Vec<_> = args.iter().map(|arg| self.eval(arg)).collect();

        let Object::Callable(function) = callee else {
            panic!("'{callee}' is not callable!")
        };

        if function.arity() != arguments.len() {
            let arity = function.arity();
            let n_args = arguments.len();
            panic!("called fn/{arity} with {n_args}");
        }
        function.call(self, &arguments)
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

    fn eval_var(&mut self, name: &Identifier) -> Object {
        self.environment
            .get(name)
            .unwrap_or_else(|| panic!("undefined variable {name}"))
            .clone()
    }

    fn eval_assign(&mut self, name: &Identifier, expr: &Expr) -> Object {
        let value = self.eval(expr);
        self.environment
            .mutate(name, value)
            .unwrap_or_else(|| panic!("undefined {name}"))
            .clone()
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
