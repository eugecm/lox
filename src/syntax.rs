use eyre::Context;
use std::{borrow::Cow, fmt::Display, iter::Peekable};

use crate::scanner::{Token, TokenType};

#[derive(Debug, Clone, PartialEq)]
pub enum Literal<'a> {
    Identifier(&'a str),
    String(Cow<'a, str>),
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

#[derive(Debug)]
pub enum Expr<'a> {
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

    pub fn parse(&'b mut self) -> Expr<'a> {
        self.expression()
    }

    fn expression(&'b mut self) -> Expr<'a> {
        self.equality()
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
                value: Literal::String(Cow::Borrowed(token.lexeme)),
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
