use std::{borrow::Cow, rc::Rc};

use crate::{
    environment::Environment,
    scanner::{Token, TokenType},
    syntax::{Expr, Literal},
};

// 'a is the lifetime bound to the AST. 'b is the lifetime bound to the source code

pub fn eval<'a, 'b: 'a>(expr: &'a Expr<'b>, env: &'a Environment<'b>) -> Literal<'b> {
    match expr {
        Expr::Binary { left, op, right } => eval_binary(left, *op, right, env),
        Expr::Grouping { expr } => eval(expr, env),
        Expr::Literal { value } => eval_literal(value),
        Expr::Unary { op, right } => eval_unary(*op, right, env),
        Expr::Var { name } => eval_var(name, env),
        Expr::Assign { name, expr } => eval_assign(name, expr, env),
    }
}

fn eval_literal<'a>(value: &Literal<'a>) -> Literal<'a> {
    value.clone()
}

fn eval_unary<'a, 'b: 'a>(op: Token, right: &Expr<'b>, env: &'a Environment<'b>) -> Literal<'b> {
    match op.typ {
        TokenType::Minus => {
            let sub = eval(right, env);
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

fn eval_binary<'a, 'b: 'a>(
    left: &Expr<'b>,
    op: Token,
    right: &Expr<'b>,
    env: &'a Environment<'b>,
) -> Literal<'b> {
    let left = eval(left, env);
    let right = eval(right, env);
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
            Literal::String(Rc::new(Cow::Owned(format!("{left}{right}"))))
        }

        (left, op, right) => {
            panic!(
                "invalid operator '{op:?}' for '{left:?}' and '{right:?}'. This isn't javascript"
            )
        }
    }
}

fn eval_var<'a, 'b: 'a>(name: &'b str, env: &'a Environment<'b>) -> Literal<'b> {
    env.get(name)
        .unwrap_or_else(|| panic!("undefined variable {name}"))
        .clone()
}

fn eval_assign<'a, 'b: 'a>(
    name: &'b str,
    expr: &'a Expr<'b>,
    env: &'a Environment<'b>,
) -> Literal<'b> {
    let value = eval(expr, env);
    env.mutate(name, value)
        .unwrap_or_else(|| panic!("undefined {name}"))
        .clone()
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
    use std::rc::Rc;

    use crate::{
        environment::Environment,
        eval,
        scanner::Scanner,
        syntax::{Declaration, Literal, Parser, Program, Stmt},
    };

    #[test]
    fn test_expressions() {
        let cases = [
            ("1+1;", Literal::Number(2.)),
            ("(1+3)*5;", Literal::Number(20.)),
            ("20 == 14;", Literal::Boolean(false)),
            ("20 != 14;", Literal::Boolean(true)),
            (r#""hello" == "hello";"#, Literal::Boolean(true)),
            (r#""hello" == "hi";"#, Literal::Boolean(false)),
            (
                r#""foo" + "bar";"#,
                Literal::String(Rc::new("foobar".into())),
            ),
        ];

        for (expr_str, expected) in cases {
            let scanner = Scanner::new(&expr_str);
            let mut parser = Parser::new(scanner.scan_tokens().map(|t| t.unwrap()));
            let ast = parser.parse();
            let Program::Declarations(decls) = ast;
            let env = Environment::default();
            for decl in decls {
                match decl {
                    Declaration::Statement(Stmt::ExprStmt(expr)) => {
                        let result = eval::eval(&expr, &env);
                        assert_eq!(expected, result);
                    }
                    _ => panic!("test can only include expressions"),
                }
            }
        }
    }
}
