use eyre::Context;
use std::{borrow::Cow, fmt::Display, iter::Peekable, rc::Rc};

use crate::scanner::{Token, TokenType};

/// The AST for the program is represented as an enum
#[derive(Debug)]
pub enum Program<'a> {
    Declarations(Vec<Declaration<'a>>),
}

#[derive(Debug)]
pub enum Declaration<'a> {
    Var {
        identifier: &'a str,
        expression: Expr<'a>,
    },
    Statement(Stmt<'a>),
}

#[derive(Debug)]
pub enum Stmt<'a> {
    ExprStmt(Expr<'a>),
    PrintStmt(Expr<'a>),
    Block(Vec<Declaration<'a>>),
}

#[derive(Debug)]
pub enum Expr<'a> {
    Assign {
        name: &'a str,
        expr: Box<Expr<'a>>,
    },
    Binary {
        left: Box<Expr<'a>>,
        op: Token<'a>,
        right: Box<Expr<'a>>,
    },
    Grouping {
        expr: Box<Expr<'a>>,
    },
    Literal {
        value: Literal<'a>,
    },
    Unary {
        op: Token<'a>,
        right: Box<Expr<'a>>,
    },
    Var {
        name: &'a str,
    },
}

impl Display for Expr<'_> {
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
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Literal<'a> {
    Identifier(&'a str),
    String(Rc<Cow<'a, str>>),
    Number(f64),
    Boolean(bool),
    Null, // eww
}

impl Display for Literal<'_> {
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
        fn $name(&'b mut self) -> Expr<'a> {
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

pub struct Parser<'a, T>
where
    T: Iterator<Item = Token<'a>>,
{
    tokens: Peekable<T>,
}

impl<'a: 'b, 'b, T> Parser<'a, T>
where
    T: Iterator<Item = Token<'a>>,
{
    pub fn new(tokens: T) -> Self {
        Self {
            tokens: tokens.peekable(),
        }
    }

    pub fn parse(&'b mut self) -> Program<'a> {
        let mut decls = Vec::new();
        while self.tokens.peek().is_some() {
            decls.push(self.declaration());
        }
        Program::Declarations(decls)
    }

    fn declaration(&'b mut self) -> Declaration<'a> {
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

    fn expression(&'b mut self) -> Expr<'a> {
        self.assignment()
    }

    fn assignment(&'b mut self) -> Expr<'a> {
        let expr = self.equality();

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

    fn statement(&'b mut self) -> Stmt<'a> {
        if self.matches(&[TokenType::Print]).is_some() {
            return self.print_statement();
        }

        if self.matches(&[TokenType::LeftBrace]).is_some() {
            return self.block();
        }

        let expr = self.expression();
        self.matches(&[TokenType::Semicolon]).expect("expected ';'");
        return Stmt::ExprStmt(expr);
    }

    fn block(&'b mut self) -> Stmt<'a> {
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

    fn print_statement(&'b mut self) -> Stmt<'a> {
        let expr = self.expression();
        self.matches(&[TokenType::Semicolon]).expect("expected ';'");
        Stmt::PrintStmt(expr)
    }

    fn matches(&'b mut self, types: &[TokenType]) -> Option<Token<'a>> {
        self.tokens.next_if(|t| types.contains(&t.typ))
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

    fn unary(&'b mut self) -> Expr<'a> {
        if let Some(op) = self.matches(&[TokenType::Bang, TokenType::Minus]) {
            return Expr::Unary {
                op,
                right: Box::new(self.unary()),
            };
        }

        self.primary()
    }

    fn primary(&'b mut self) -> Expr<'a> {
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
                value: Literal::String(Rc::new(Cow::Borrowed(token.lexeme))),
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
                name: &token.lexeme,
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
