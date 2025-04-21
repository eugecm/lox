use eyre::Context;
use std::{fmt::Display, iter::Peekable};

use crate::{
    scanner::{Token, TokenType},
    types::{Identifier, Object},
};

/// The AST for the program is represented as an enum
#[derive(Debug)]
pub enum Program {
    Declarations(Vec<Declaration>),
}

#[derive(Debug, Clone)]
pub enum Declaration {
    Var {
        identifier: Identifier,
        expression: Expr,
    },
    Statement(Stmt),
}

#[derive(Debug, Clone)]
pub struct FunctionStmt {
    pub identifier: Identifier,
    pub parameters: Vec<Token>,
    pub body: Vec<Declaration>,
}

#[derive(Debug, Clone)]
pub struct ClassDecl {
    pub name: Identifier,
    pub methods: Vec<FunctionStmt>,
    pub superclass: Option<Expr>,
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Expr(Expr),
    FunctionDecl(FunctionStmt),
    ClassDecl(ClassDecl),
    If {
        condition: Expr,
        then_branch: Box<Stmt>,
        else_branch: Option<Box<Stmt>>,
    },
    Print(Expr),
    Return {
        value: Expr,
    },
    While {
        condition: Expr,
        body: Box<Stmt>,
    },
    Block(Vec<Declaration>),
}

#[derive(Debug, Clone)]
pub struct Expr {
    /// Unique identifier for this expr in the AST
    pub id: u64,
    pub kind: ExprKind,
}

#[derive(Debug, Clone)]
pub enum ExprKind {
    Assign {
        name: Identifier,
        expr: Box<Expr>,
    },
    Binary {
        left: Box<Expr>,
        op: Token,
        right: Box<Expr>,
    },
    Grouping {
        expr: Box<Expr>,
    },
    Literal {
        value: Object,
    },
    Logical {
        left: Box<Expr>,
        op: Token,
        right: Box<Expr>,
    },
    Unary {
        op: Token,
        right: Box<Expr>,
    },
    Call {
        callee: Box<Expr>,
        #[allow(dead_code)]
        parens: Token,
        args: Vec<Expr>,
    },
    Get {
        name: Identifier,
        object: Box<Expr>,
    },
    Set {
        object: Box<Expr>,
        name: Identifier,
        value: Box<Expr>,
    },
    This {
        token: Identifier,
    },
    Var {
        name: Identifier,
    },
}

impl Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            ExprKind::Binary { left, op, right } => {
                Display::fmt(&left, f)?;
                Display::fmt(&op, f)?;
                Display::fmt(&right, f)?;
            }
            ExprKind::Grouping { expr } => {
                write!(f, "(")?;
                Display::fmt(&expr, f)?;
                write!(f, ")")?;
            }
            ExprKind::Literal { value } => Display::fmt(value, f)?,
            ExprKind::Unary { op, right } => {
                Display::fmt(op, f)?;
                Display::fmt(right, f)?;
            }
            ExprKind::Var { name } => Display::fmt(name, f)?,
            ExprKind::Assign { name, expr } => {
                Display::fmt(name, f)?;
                write!(f, "=")?;
                Display::fmt(&expr, f)?;
            }
            ExprKind::Logical { left, op, right } => {
                Display::fmt(&left, f)?;
                Display::fmt(&op, f)?;
                Display::fmt(&right, f)?;
            }
            ExprKind::Call {
                callee,
                parens: _,
                args,
            } => {
                Display::fmt(&callee, f)?;
                write!(f, "(")?;
                for (i, arg) in args.iter().enumerate() {
                    Display::fmt(&arg, f)?;
                    if i != args.len() - 1 {
                        write!(f, ",")?;
                    }
                }
                write!(f, ")")?;
            }
            ExprKind::Get { name, object } => {
                write!(f, "{object}.{name}")?;
            }
            ExprKind::Set {
                object,
                name,
                value,
            } => {
                write!(f, "{object}.{name}={value}")?;
            }
            ExprKind::This { token: _ } => write!(f, "this")?,
        }
        Ok(())
    }
}

macro_rules! binary_expr {
    ( $name:ident, $left:ident, $ops:expr, $right:ident ) => {
        fn $name(&mut self) -> Expr {
            let mut expr = self.$left();

            while let Some(op) = self.matches($ops) {
                let right = self.$right();
                let id = self.get_expr_id();
                expr = Expr {
                    id,
                    kind: ExprKind::Binary {
                        left: Box::new(expr),
                        op,
                        right: Box::new(right),
                    },
                }
            }

            expr
        }
    };
}

pub struct Parser<T>
where
    T: Iterator<Item = Token>,
{
    tokens: Peekable<T>,
    expr_counter: u64,
}

impl<T> Parser<T>
where
    T: Iterator<Item = Token>,
{
    pub fn new(tokens: T) -> Self {
        Self {
            tokens: tokens.peekable(),
            expr_counter: 0,
        }
    }

    fn get_expr_id(&mut self) -> u64 {
        let old = self.expr_counter;
        self.expr_counter += 1;
        old
    }

    pub fn parse(&mut self) -> Program {
        let mut decls = Vec::new();
        while self.tokens.peek().is_some() {
            decls.push(self.declaration());
        }
        Program::Declarations(decls)
    }

    fn declaration(&mut self) -> Declaration {
        if let Some(t) = self.matches(&[TokenType::Var]) {
            let Some(name) = self.matches(&[TokenType::Identifier]) else {
                panic!("expected identifier on line {}", t.line);
            };
            // All variables must be initialized
            self.matches(&[TokenType::Equal])
                .unwrap_or_else(|| panic!("expected '=' after VAR on line {}", t.line));
            let initializer = self.expression();
            self.matches(&[TokenType::Semicolon]).expect("expected ';'");
            Declaration::Var {
                identifier: Identifier(name.lexeme.into()),
                expression: initializer,
            }
        } else if let Some(_) = self.matches(&[TokenType::Fun]) {
            Declaration::Statement(self.function("function"))
        } else if let Some(_) = self.matches(&[TokenType::Class]) {
            Declaration::Statement(self.class())
        } else {
            let stmt = self.statement();
            Declaration::Statement(stmt)
        }
    }

    fn class(&mut self) -> Stmt {
        let Some(name) = self.matches(&[TokenType::Identifier]) else {
            panic!("invalid syntax: expected identifier")
        };

        let superclass = self.matches(&[TokenType::Less]).map(|_| {
            let Some(name) = self.matches(&[TokenType::Identifier]) else {
                panic!("invalid syntax: expected superclass name")
            };

            Expr {
                id: self.get_expr_id(),
                kind: ExprKind::Var {
                    name: Identifier(name.lexeme),
                },
            }
        });

        let _ = self
            .matches(&[TokenType::LeftBrace])
            .unwrap_or_else(|| panic!("Expected '{{' after class name"));

        let mut methods = Vec::new();
        while !self.peek_matches(&[TokenType::RightBrace]) {
            let Stmt::FunctionDecl(decl) = self.function("method") else {
                panic!("bug: 'function' can only return function declarations")
            };
            methods.push(decl);
        }

        let _ = self
            .matches(&[TokenType::RightBrace])
            .unwrap_or_else(|| panic!("Expected '}}' after class body"));

        Stmt::ClassDecl(ClassDecl {
            name: Identifier(name.lexeme.clone()),
            methods,
            superclass,
        })
    }

    fn expression(&mut self) -> Expr {
        self.assignment()
    }

    fn assignment(&mut self) -> Expr {
        let expr = self.or();

        if self.matches(&[TokenType::Equal]).is_some() {
            let value = self.assignment();

            match expr.kind {
                ExprKind::Var { name } => {
                    return Expr {
                        id: self.get_expr_id(),
                        kind: ExprKind::Assign {
                            name,
                            expr: Box::new(value),
                        },
                    };
                }
                ExprKind::Get { object, name } => {
                    return Expr {
                        id: self.get_expr_id(),
                        kind: ExprKind::Set {
                            object,
                            name,
                            value: Box::new(value),
                        },
                    }
                }
                _ => panic!("Invalid assignment target"),
            }
        }

        expr
    }

    fn or(&mut self) -> Expr {
        let mut expr = self.and();

        while let Some(t) = self.matches(&[TokenType::Or]) {
            let op = t;
            let right = self.and();
            expr = Expr {
                id: self.get_expr_id(),
                kind: ExprKind::Logical {
                    left: Box::new(expr),
                    op,
                    right: Box::new(right),
                },
            }
        }

        expr
    }

    fn and(&mut self) -> Expr {
        let mut expr = self.equality();

        while let Some(t) = self.matches(&[TokenType::And]) {
            let op = t;
            let right = self.equality();
            expr = Expr {
                id: self.get_expr_id(),
                kind: ExprKind::Logical {
                    left: Box::new(expr),
                    op,
                    right: Box::new(right),
                },
            }
        }

        expr
    }

    fn statement(&mut self) -> Stmt {
        if self.matches(&[TokenType::If]).is_some() {
            return self.if_statement();
        }

        if self.matches(&[TokenType::Print]).is_some() {
            return self.print_statement();
        }

        if self.matches(&[TokenType::Return]).is_some() {
            return self.return_statement();
        }

        if self.matches(&[TokenType::For]).is_some() {
            return self.for_statement();
        }

        if self.matches(&[TokenType::While]).is_some() {
            return self.while_statement();
        }

        if self.matches(&[TokenType::LeftBrace]).is_some() {
            return self.block();
        }

        let expr = self.expression();
        self.matches(&[TokenType::Semicolon]).expect("expected ';'");
        Stmt::Expr(expr)
    }

    fn return_statement(&mut self) -> Stmt {
        let mut value = Expr {
            id: self.get_expr_id(),
            kind: ExprKind::Literal {
                value: Object::Null,
            },
        };
        if !self.peek_matches(&[TokenType::Semicolon]) {
            value = self.expression();
        }

        self.matches(&[TokenType::Semicolon]).expect("expected ';'");
        Stmt::Return { value }
    }

    fn for_statement(&mut self) -> Stmt {
        self.matches(&[TokenType::LeftParen])
            .expect("expected '(' after 'for'");

        let initializer = if self.matches(&[TokenType::Semicolon]).is_some() {
            None
        } else if self.peek_matches(&[TokenType::Var]) {
            Some(self.declaration())
        } else {
            Some(Declaration::Statement(Stmt::Expr(self.expression())))
        };

        let condition = if let Some(token) = self.tokens.peek() {
            if token.typ != TokenType::Semicolon {
                Some(self.expression())
            } else {
                None
            }
        } else {
            None
        };

        self.matches(&[TokenType::Semicolon])
            .expect("expected ';' after loop condition");

        let increment = if let Some(token) = self.tokens.peek() {
            if token.typ != TokenType::RightParen {
                Some(self.expression())
            } else {
                None
            }
        } else {
            None
        };
        self.matches(&[TokenType::RightParen])
            .expect("expected ')' after for clauses");

        let mut body = self.statement();

        body = if let Some(increment) = increment {
            Stmt::Block(vec![
                Declaration::Statement(body),
                Declaration::Statement(Stmt::Expr(increment)),
            ])
        } else {
            body
        };

        let condition = condition.unwrap_or(Expr {
            id: self.get_expr_id(),
            kind: ExprKind::Literal {
                value: Object::Boolean(true),
            },
        });

        body = Stmt::While {
            condition,
            body: Box::new(body),
        };

        if let Some(initializer) = initializer {
            body = Stmt::Block(vec![initializer, Declaration::Statement(body)]);
        }

        body
    }

    fn while_statement(&mut self) -> Stmt {
        self.matches(&[TokenType::LeftParen])
            .expect("expected '(' after 'while'");
        let condition = self.expression();
        self.matches(&[TokenType::RightParen])
            .expect("expected ')' after while condition");
        let body = self.statement();

        Stmt::While {
            condition,
            body: Box::new(body),
        }
    }

    fn block(&mut self) -> Stmt {
        let mut statements = Vec::new();
        loop {
            let Some(next) = self.tokens.peek() else {
                break;
            };
            if next.typ == TokenType::RightBrace {
                break;
            };
            statements.push(self.declaration());
        }

        self.matches(&[TokenType::RightBrace])
            .unwrap_or_else(|| panic!("expected '}}'"));
        Stmt::Block(statements)
    }

    fn if_statement(&mut self) -> Stmt {
        self.matches(&[TokenType::LeftParen])
            .expect("expected '(' after 'if'");
        let condition = self.expression();
        self.matches(&[TokenType::RightParen])
            .expect("expected ')' after if condition");

        let then_branch = Box::new(self.statement());
        let else_branch = self
            .matches(&[TokenType::Else])
            .map(|_| Box::new(self.statement()));

        Stmt::If {
            condition,
            then_branch,
            else_branch,
        }
    }

    fn print_statement(&mut self) -> Stmt {
        let expr = self.expression();
        self.matches(&[TokenType::Semicolon]).expect("expected ';'");
        Stmt::Print(expr)
    }

    fn matches(&mut self, types: &[TokenType]) -> Option<Token> {
        self.tokens.next_if(|t| types.contains(&t.typ))
    }

    fn peek_matches(&mut self, types: &[TokenType]) -> bool {
        self.tokens
            .peek()
            .map(|t| types.contains(&t.typ))
            .unwrap_or_default()
    }

    binary_expr!(
        equality,
        comparison,
        &[TokenType::BangEqual, TokenType::EqualEqual],
        comparison
    );
    binary_expr!(
        comparison,
        term,
        &[
            TokenType::Greater,
            TokenType::GreaterEqual,
            TokenType::Less,
            TokenType::LessEqual,
        ],
        term
    );
    binary_expr!(term, factor, &[TokenType::Minus, TokenType::Plus], factor);
    binary_expr!(factor, unary, &[TokenType::Slash, TokenType::Star], unary);

    fn unary(&mut self) -> Expr {
        if let Some(op) = self.matches(&[TokenType::Bang, TokenType::Minus]) {
            return Expr {
                id: self.get_expr_id(),
                kind: ExprKind::Unary {
                    op,
                    right: Box::new(self.unary()),
                },
            };
        }

        self.call()
    }

    fn call(&mut self) -> Expr {
        let mut expr = self.primary();

        loop {
            if self.matches(&[TokenType::LeftParen]).is_some() {
                expr = self.finish_call(expr);
            } else if self.matches(&[TokenType::Dot]).is_some() {
                let Some(name) = self.matches(&[TokenType::Identifier]) else {
                    panic!("Expect property name after '.'")
                };
                expr = Expr {
                    id: self.get_expr_id(),
                    kind: ExprKind::Get {
                        name: Identifier(name.lexeme.clone()),
                        object: Box::new(expr),
                    },
                }
            } else {
                break;
            }
        }

        expr
    }

    fn finish_call(&mut self, callee: Expr) -> Expr {
        let mut args = Vec::new();

        if !self.peek_matches(&[TokenType::RightParen]) {
            args.push(self.expression());
            while self.matches(&[TokenType::Comma]).is_some() {
                args.push(self.expression())
            }
        }

        if args.len() > 255 {
            panic!("can't have more than 255 arguments!")
        }

        let tok = self
            .matches(&[TokenType::RightParen])
            .unwrap_or_else(|| panic!("expected right paren in function call"));

        Expr {
            id: self.get_expr_id(),
            kind: ExprKind::Call {
                callee: Box::new(callee),
                parens: tok,
                args,
            },
        }
    }

    fn primary(&mut self) -> Expr {
        let token = self.tokens.next().expect("unexpected end of token stream");
        match token.typ {
            TokenType::False => Expr {
                id: self.get_expr_id(),
                kind: ExprKind::Literal {
                    value: Object::Boolean(false),
                },
            },
            TokenType::True => Expr {
                id: self.get_expr_id(),
                kind: ExprKind::Literal {
                    value: Object::Boolean(true),
                },
            },
            TokenType::Nil => Expr {
                id: self.get_expr_id(),
                kind: ExprKind::Literal {
                    value: Object::Null,
                },
            },
            TokenType::Number => Expr {
                id: self.get_expr_id(),
                kind: ExprKind::Literal {
                    value: Object::Number(
                        token
                            .lexeme
                            .parse()
                            .with_context(|| format!("parsing number {token:?}"))
                            .unwrap(),
                    ),
                },
            },
            TokenType::String => Expr {
                id: self.get_expr_id(),
                kind: ExprKind::Literal {
                    value: Object::String(token.lexeme),
                },
            },

            TokenType::LeftParen => {
                let expr = self.expression();
                let Some(right_parens) = self.tokens.next() else {
                    panic!("expected ')' but found no tokens")
                };

                if right_parens.typ != TokenType::RightParen {
                    panic!("expected ')' but found {right_parens}")
                }

                Expr {
                    id: self.get_expr_id(),
                    kind: ExprKind::Grouping {
                        expr: Box::new(expr),
                    },
                }
            }
            TokenType::Identifier => Expr {
                id: self.get_expr_id(),
                kind: ExprKind::Var {
                    name: Identifier(token.lexeme),
                },
            },
            TokenType::This => Expr {
                id: self.get_expr_id(),
                kind: ExprKind::This {
                    token: Identifier(token.lexeme),
                },
            },
            _ => panic!("primary: unexpected token {token:?}"),
        }
    }

    fn function(&mut self, kind: &str) -> Stmt {
        let name = self
            .matches(&[TokenType::Identifier])
            .unwrap_or_else(|| panic!("Expected {kind} name."));
        let _ = self
            .matches(&[TokenType::LeftParen])
            .unwrap_or_else(|| panic!("Expected '(' after {kind} name"));

        let mut parameters = Vec::new();
        if !self.peek_matches(&[TokenType::RightParen]) {
            loop {
                if parameters.len() > 255 {
                    panic!("can't define function with more than 255 params");
                }

                parameters.push(
                    self.matches(&[TokenType::Identifier])
                        .expect("Expected parameter name"),
                );
                if self.matches(&[TokenType::Comma]).is_none() {
                    break;
                }
            }
        }
        let _ = self
            .matches(&[TokenType::RightParen])
            .expect("Expected ')' after parameters");

        // Now consume the body
        let _ = self
            .matches(&[TokenType::LeftBrace])
            .unwrap_or_else(|| panic!("Expected '{{' before {kind} body"));
        let Stmt::Block(body) = self.block() else {
            panic!("block should only return Stmt::Block")
        };

        Stmt::FunctionDecl(FunctionStmt {
            identifier: Identifier(name.lexeme),
            parameters,
            body,
        })
    }
}

#[test]
fn test() {
    use crate::scanner::TokenType;

    let expr = Expr {
        id: 0,
        kind: ExprKind::Binary {
            left: Box::new(Expr {
                id: 1,
                kind: ExprKind::Literal {
                    value: Object::Number(1.2),
                },
            }),
            op: Token::new(TokenType::Plus, "+", 0),
            right: Box::new(Expr {
                id: 2,
                kind: ExprKind::Literal {
                    value: Object::Number(3.4),
                },
            }),
        },
    };
    println!("{expr}");
}
