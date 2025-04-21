use std::collections::HashMap;

use crate::{
    interpreter::Interpreter,
    syntax::{Declaration, Expr, ExprKind, Program, Stmt},
    types::Identifier,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FunctionType {
    None,
    Function,
    Method,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ClassType {
    None,
    Class,
}

#[derive(Debug)]
pub struct Resolver {
    interpreter: Interpreter,
    scopes: Vec<HashMap<Identifier, bool>>,
    current_function: FunctionType,
    current_class: ClassType,
}

impl Resolver {
    pub fn new(interpreter: Interpreter) -> Self {
        Self {
            interpreter,
            scopes: Default::default(),
            current_function: FunctionType::None,
            current_class: ClassType::None,
        }
    }
    pub fn run(mut self, prog: &Program) -> Interpreter {
        match prog {
            Program::Declarations(decls) => {
                self.resolve(&decls);
            }
        }
        self.interpreter
    }

    pub fn resolve(&mut self, stmts: &[Declaration]) {
        for stmt in stmts {
            match stmt {
                Declaration::Var {
                    identifier,
                    expression,
                } => {
                    self.declare(identifier.clone());
                    self.resolve_expr(expression);
                    self.define(identifier.clone());
                }
                Declaration::Statement(stmt) => {
                    self.resolve_stmt(stmt);
                }
            }
        }
    }

    fn resolve_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Expr(expr) => {
                self.resolve_expr(expr);
            }
            Stmt::FunctionDecl(function_stmt) => {
                self.declare(function_stmt.identifier.clone());
                self.define(function_stmt.identifier.clone());
                self.resolve_function(function_stmt, FunctionType::Function);
            }
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                self.resolve_expr(condition);
                self.resolve_stmt(then_branch);
                if let Some(else_branch) = else_branch {
                    self.resolve_stmt(else_branch);
                }
            }
            Stmt::Print(expr) => {
                self.resolve_expr(expr);
            }
            Stmt::Return { value } => {
                if self.current_function == FunctionType::None {
                    panic!("can't return from a top-level function")
                }

                self.resolve_expr(value);
            }
            Stmt::While { condition, body } => {
                self.resolve_expr(condition);
                self.resolve_stmt(body);
            }
            Stmt::Block(declarations) => {
                self.begin_scope();
                self.resolve(declarations);
                self.end_scope();
            }
            Stmt::ClassDecl(class_decl) => {
                let enclosing_class = self.current_class;
                self.current_class = ClassType::Class;

                self.declare(class_decl.name.clone());
                self.define(class_decl.name.clone());

                self.begin_scope();
                self.scopes
                    .last_mut()
                    .unwrap()
                    .insert(Identifier("this".into()), true);

                for method in &class_decl.methods {
                    self.resolve_function(method, FunctionType::Method);
                }

                self.end_scope();
                self.current_class = enclosing_class;
            }
        }
    }

    fn resolve_expr(&mut self, expr: &Expr) {
        match &expr.kind {
            ExprKind::Assign { name, expr: child } => {
                self.resolve_expr(child);
                self.resolve_local(expr, name);
            }
            ExprKind::Binary { left, op: _, right } => {
                self.resolve_expr(left);
                self.resolve_expr(right);
            }
            ExprKind::Grouping { expr } => {
                self.resolve_expr(expr);
            }
            ExprKind::Literal { value: _ } => {}
            ExprKind::Logical { left, op: _, right } => {
                self.resolve_expr(left);
                self.resolve_expr(right);
            }
            ExprKind::Unary { op: _, right } => {
                self.resolve_expr(right);
            }
            ExprKind::Call {
                callee,
                parens: _,
                args,
            } => {
                self.resolve_expr(callee);
                for arg in args {
                    self.resolve_expr(arg);
                }
            }
            ExprKind::Var { name } => {
                if let Some(false) = self.scopes.last().map(|s| s.get(name).unwrap_or(&true)) {
                    panic!("can't read local var in its own initializer");
                }
                self.resolve_local(expr, name);
            }
            ExprKind::Get { name: _, object } => {
                self.resolve_expr(object);
            }
            ExprKind::Set {
                object,
                name: _,
                value,
            } => {
                self.resolve_expr(value);
                self.resolve_expr(object);
            }
            ExprKind::This { token } => {
                if self.current_class == ClassType::None {
                    panic!("can't use 'this' keyword outside of a class");
                }

                self.resolve_local(expr, token);
            }
        }
    }

    fn begin_scope(&mut self) {
        self.scopes.push(HashMap::default());
    }

    fn end_scope(&mut self) {
        self.scopes.pop().expect("stack is empty!");
    }

    fn declare(&mut self, name: Identifier) {
        if let Some(scope) = self.scopes.last_mut() {
            if scope.contains_key(&name) {
                panic!("Already a variable with this name in this scope. {scope:?}");
            }
            scope.insert(name, false);
        }
    }

    fn define(&mut self, name: Identifier) {
        self.scopes
            .last_mut()
            .and_then(|scope| scope.insert(name, true));
    }

    fn resolve_local(&mut self, expr: &Expr, name: &Identifier) {
        for (i, scope) in self.scopes.iter().enumerate().rev() {
            if scope.contains_key(name) {
                let depth = self.scopes.len() - 1 - i;
                self.interpreter.resolve(expr, depth);
                return;
            }
        }
    }

    fn resolve_function(
        &mut self,
        function_stmt: &crate::syntax::FunctionStmt,
        kind: FunctionType,
    ) {
        let enclosing_function = self.current_function;
        self.current_function = kind;

        self.begin_scope();
        for param in &function_stmt.parameters {
            let ident = Identifier(param.lexeme.clone());
            self.declare(ident.clone());
            self.define(ident);
        }
        self.resolve(&function_stmt.body);
        self.end_scope();
        self.current_function = enclosing_function;
    }
}
