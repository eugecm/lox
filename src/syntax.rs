use eyre::Context;
use std::{fmt::Display, iter::Peekable, rc::Rc};

use crate::scanner::{Token, TokenType};

/// The AST for the program is represented as an enum
#[derive(Debug)]
pub enum Program {
    Declarations(Vec<Declaration>),
}

#[derive(Debug, Clone)]
pub enum Declaration {
    Var {
        identifier: Rc<str>,
        expression: Expr,
    },
    Statement(Stmt),
}

#[derive(Debug, Clone)]
pub enum Stmt {
    ExprStmt(Expr),
    IfStmt {
        condition: Expr,
        then_branch: Box<Stmt>,
        else_branch: Option<Box<Stmt>>,
    },
    PrintStmt(Expr),
    WhileStmt {
        condition: Expr,
        body: Box<Stmt>,
    },
    Block(Vec<Declaration>),
}

#[derive(Debug, Clone)]
pub enum Expr {
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
        value: Literal,
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
    Var {
        name: Identifier,
    },
}

impl Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expr::Binary { left, op, right } => {
                Display::fmt(&left, f)?;
                Display::fmt(&op, f)?;
                Display::fmt(&right, f)?;
            }
            Expr::Grouping { expr } => {
                write!(f, "(")?;
                Display::fmt(&expr, f)?;
                write!(f, ")")?;
            }
            Expr::Literal { value } => Display::fmt(value, f)?,
            Expr::Unary { op, right } => {
                Display::fmt(op, f)?;
                Display::fmt(right, f)?;
            }
            Expr::Var { name } => Display::fmt(name, f)?,
            Expr::Assign { name, expr } => {
                Display::fmt(name, f)?;
                write!(f, "=")?;
                Display::fmt(&expr, f)?;
            }
            Expr::Logical { left, op, right } => {
                Display::fmt(&left, f)?;
                Display::fmt(&op, f)?;
                Display::fmt(&right, f)?;
            }
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

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    #[allow(dead_code)]
    Identifier(Identifier),
    String(Rc<str>),
    Number(f64),
    Boolean(bool),
    Null, // eww
}

impl Literal {
    pub fn is_truthy(&self) -> bool {
        match self {
            Literal::Boolean(value) => *value,
            typ => panic!("invalid non-boolean value {typ:?} evaluated to truthy"),
        }
    }
}

impl Display for Literal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Literal::Identifier(i) => write!(f, "{i}")?,
            Literal::String(s) => write!(f, "{s}")?,
            Literal::Number(n) => write!(f, "{n}")?,
            Literal::Boolean(n) => write!(f, "{n}")?,
            Literal::Null => write!(f, "null")?,
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
                expr = Expr::Binary {
                    left: Box::new(expr),
                    op,
                    right: Box::new(right),
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
}

impl<T> Parser<T>
where
    T: Iterator<Item = Token>,
{
    pub fn new(tokens: T) -> Self {
        Self {
            tokens: tokens.peekable(),
        }
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
                identifier: name.lexeme,
                expression: initializer,
            }
        } else {
            let stmt = self.statement();
            Declaration::Statement(stmt)
        }
    }

    fn expression(&mut self) -> Expr {
        self.assignment()
    }

    fn assignment(&mut self) -> Expr {
        let expr = self.or();

        if let Some(_) = self.matches(&[TokenType::Equal]) {
            let value = self.assignment();

            if let Expr::Var { name } = expr {
                return Expr::Assign {
                    name,
                    expr: Box::new(value),
                };
            }

            panic!("Invalid assignment target");
        }

        expr
    }

    fn or(&mut self) -> Expr {
        let mut expr = self.and();

        while let Some(t) = self.matches(&[TokenType::Or]) {
            let op = t;
            let right = self.and();
            expr = Expr::Logical {
                left: Box::new(expr),
                op,
                right: Box::new(right),
            }
        }

        expr
    }

    fn and(&mut self) -> Expr {
        let mut expr = self.equality();

        while let Some(t) = self.matches(&[TokenType::And]) {
            let op = t;
            let right = self.equality();
            expr = Expr::Logical {
                left: Box::new(expr),
                op,
                right: Box::new(right),
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
        return Stmt::ExprStmt(expr);
    }

    fn for_statement(&mut self) -> Stmt {
        self.matches(&[TokenType::LeftParen])
            .expect("expected '(' after 'for'");

        let initializer = if let Some(_) = self.matches(&[TokenType::Semicolon]) {
            None
        } else if self.peek_matches(&[TokenType::Var]) {
            Some(self.declaration())
        } else {
            Some(Declaration::Statement(Stmt::ExprStmt(self.expression())))
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
                Declaration::Statement(Stmt::ExprStmt(increment)),
            ])
        } else {
            body
        };

        let condition = condition.unwrap_or(Expr::Literal {
            value: Literal::Boolean(true),
        });

        body = Stmt::WhileStmt {
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

        Stmt::WhileStmt {
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

        Stmt::IfStmt {
            condition,
            then_branch,
            else_branch,
        }
    }

    fn print_statement(&mut self) -> Stmt {
        let expr = self.expression();
        self.matches(&[TokenType::Semicolon]).expect("expected ';'");
        Stmt::PrintStmt(expr)
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
            return Expr::Unary {
                op,
                right: Box::new(self.unary()),
            };
        }

        self.primary()
    }

    fn primary(&mut self) -> Expr {
        let token = self.tokens.next().expect("unexpected end of token stream");
        match token.typ {
            TokenType::False => Expr::Literal {
                value: Literal::Boolean(false),
            },
            TokenType::True => Expr::Literal {
                value: Literal::Boolean(true),
            },
            TokenType::Nil => Expr::Literal {
                value: Literal::Null,
            },

            TokenType::Number => Expr::Literal {
                value: Literal::Number(
                    token
                        .lexeme
                        .parse()
                        .with_context(|| format!("parsing number {token:?}"))
                        .unwrap(),
                ),
            },
            TokenType::String => Expr::Literal {
                value: Literal::String(token.lexeme),
            },

            TokenType::LeftParen => {
                let expr = self.expression();
                let Some(right_parens) = self.tokens.next() else {
                    panic!("expected ')' but found no tokens")
                };

                if right_parens.typ != TokenType::RightParen {
                    panic!("expected ')' but found {right_parens}")
                }

                Expr::Grouping {
                    expr: Box::new(expr),
                }
            }
            TokenType::Identifier => Expr::Var {
                name: Identifier(token.lexeme),
            },
            _ => panic!("primary: unexpected token {token:?}"),
        }
    }
}

#[test]
fn test() {
    use crate::scanner::TokenType;

    let expr = Expr::Binary {
        left: Box::new(Expr::Literal {
            value: Literal::Number(1.2),
        }),
        op: Token::new(TokenType::Plus, "+", 0),
        right: Box::new(Expr::Literal {
            value: Literal::Number(3.4),
        }),
    };
    println!("{expr}");
}
