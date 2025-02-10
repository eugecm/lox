use std::borrow::Cow;

use crate::{
    scanner::{Token, TokenType},
    syntax::{Expr, Literal},
};

pub fn eval<'a>(expr: &'a Expr) -> Literal<'a> {
    match expr {
        Expr::Binary { left, op, right } => eval_binary(left, *op, right),
        Expr::Grouping { expr } => eval(expr),
        Expr::Literal { value } => eval_literal(value),
        Expr::Unary { op, right } => eval_unary(*op, right),
    }
}

fn eval_literal<'a>(value: &Literal<'a>) -> Literal<'a> {
    value.clone()
}

fn eval_unary<'a>(op: Token, right: &Expr) -> Literal<'a> {
    match op.typ {
        TokenType::Minus => {
            let sub = eval(right);
            match sub {
                Literal::Number(n) => Literal::Number(-n),
                _ => panic!("invalid "),
            }
        }
        t => {
            panic!("unexpected token {t:?}. Expecting '-'")
        }
    }
}

fn eval_binary<'a>(left: &Expr, op: Token, right: &Expr) -> Literal<'a> {
    let left = eval(left);
    let right = eval(right);
    match (left, op.typ, right) {
        // Numbers
        (Literal::Number(left), TokenType::Minus, Literal::Number(right)) => {
            Literal::Number(left - right)
        }
        (Literal::Number(left), TokenType::Plus, Literal::Number(right)) => {
            Literal::Number(left + right)
        }
        (Literal::Number(left), TokenType::Slash, Literal::Number(right)) => {
            Literal::Number(left / right)
        }
        (Literal::Number(left), TokenType::Star, Literal::Number(right)) => {
            Literal::Number(left * right)
        }
        (Literal::Number(left), TokenType::Greater, Literal::Number(right)) => {
            Literal::Boolean(left > right)
        }
        (Literal::Number(left), TokenType::GreaterEqual, Literal::Number(right)) => {
            Literal::Boolean(left >= right)
        }
        (Literal::Number(left), TokenType::Less, Literal::Number(right)) => {
            Literal::Boolean(left < right)
        }
        (Literal::Number(left), TokenType::LessEqual, Literal::Number(right)) => {
            Literal::Boolean(left <= right)
        }
        (left, TokenType::EqualEqual, right) => Literal::Boolean(is_equal(left, right)),
        (left, TokenType::BangEqual, right) => Literal::Boolean(!is_equal(left, right)),

        (Literal::String(left), TokenType::Plus, Literal::String(right)) => {
            Literal::String(Cow::Owned(format!("{left}{right}")))
        }

        (left, op, right) => {
            panic!(
                "invalid operator '{op:?}' for '{left:?}' and '{right:?}'. This isn't javascript"
            )
        }
    }
}

fn is_equal(left: Literal, right: Literal) -> bool {
    match (left, right) {
        (Literal::String(left), Literal::String(right)) => left == right,
        (Literal::Number(left), Literal::Number(right)) => left == right,
        (Literal::Boolean(left), Literal::Boolean(right)) => left == right,
        (Literal::Null, Literal::Null) => true,
        _ => false,
    }
}

#[cfg(test)]
mod test {
    use crate::{
        scanner::Scanner,
        syntax::{Literal, Parser},
    };

    use super::eval;

    #[test]
    fn test_expressions() {
        let cases = [
            ("1+1", Literal::Number(2.)),
            ("(1+3)*5", Literal::Number(20.)),
            ("20 == 14", Literal::Boolean(false)),
            ("20 != 14", Literal::Boolean(true)),
            (r#""hello" == "hello""#, Literal::Boolean(true)),
            (r#""hello" == "hi""#, Literal::Boolean(false)),
            (r#""foo" + "bar""#, Literal::String("foobar".into())),
        ];

        for (expr_str, expected) in cases {
            let scanner = Scanner::new(&expr_str);
            let mut parser = Parser::new(scanner.scan_tokens().map(|t| t.unwrap()));
            let ast = parser.parse();
            let result = eval(&ast);
            assert_eq!(expected, result);
        }
    }
}
