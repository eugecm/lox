use std::fmt::Display;

use crate::scanner::Token;

#[derive(Debug)]
pub enum Literal<'a> {
    Identifier(&'a str),
    String(&'a str),
    Number(f64),
}

impl Display for Literal<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Literal::Identifier(i) => write!(f, "{i}")?,
            Literal::String(s) => write!(f, "{s}")?,
            Literal::Number(n) => write!(f, "{n}")?,
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
