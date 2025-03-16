use crate::{
    environment::Environment,
    scanner::{Token, TokenType},
    syntax::Expr,
    types::{Identifier, Object},
};

// 'a is the lifetime bound to the AST. 'b is the lifetime bound to the source code

pub fn eval(expr: &Expr, env: &mut Environment) -> Object {
    match expr {
        Expr::Binary { left, op, right } => eval_binary(left, op, right, env),
        Expr::Grouping { expr } => eval(expr, env),
        Expr::Literal { value } => eval_literal(value),
        Expr::Unary { op, right } => eval_unary(op, right, env),
        Expr::Var { name } => eval_var(name, env),
        Expr::Assign { name, expr } => eval_assign(name, expr, env),
        Expr::Logical { left, op, right } => eval_logical(left, op, right, env),
        Expr::Call {
            callee,
            parens: _,
            args,
        } => eval_call(callee, args, env),
    }
}

fn eval_call(callee: &Expr, args: &[Expr], env: &mut Environment) -> Object {
    let callee = eval(callee, env);

    let arguments: Vec<_> = args.iter().map(|arg| eval(arg, env)).collect();

    let Object::Callable(function) = callee else {
        panic!("'{callee}' is not callable!")
    };

    if function.arity() != arguments.len() {
        let arity = function.arity();
        let n_args = arguments.len();
        panic!("called fn/{arity} with {n_args}");
    }
    function.call(&arguments)
}

fn eval_logical(left: &Expr, op: &Token, right: &Expr, env: &mut Environment) -> Object {
    let left = eval(left, env);

    if op.typ == TokenType::Or {
        if left.is_truthy() {
            return left;
        }
    } else if !left.is_truthy() {
        return left;
    }

    eval(right, env)
}

fn eval_literal(value: &Object) -> Object {
    value.clone()
}

fn eval_unary(op: &Token, right: &Expr, env: &mut Environment) -> Object {
    match op.typ {
        TokenType::Minus => {
            let sub = eval(right, env);
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

fn eval_binary(left: &Expr, op: &Token, right: &Expr, env: &mut Environment) -> Object {
    let left = eval(left, env);
    let right = eval(right, env);
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

fn eval_var(name: &Identifier, env: &Environment) -> Object {
    env.get(name)
        .unwrap_or_else(|| panic!("undefined variable {name}"))
        .clone()
}

fn eval_assign(name: &Identifier, expr: &Expr, env: &mut Environment) -> Object {
    let value = eval(expr, env);
    env.mutate(name, value)
        .unwrap_or_else(|| panic!("undefined {name}"))
        .clone()
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

#[cfg(test)]
mod test {
    use crate::{
        environment::Environment,
        eval,
        scanner::Scanner,
        syntax::{Declaration, Parser, Program, Stmt},
        types::Object,
    };

    #[test]
    fn test_expressions() {
        let cases = [
            ("1+1;", Object::Number(2.)),
            ("(1+3)*5;", Object::Number(20.)),
            ("20 == 14;", Object::Boolean(false)),
            ("20 != 14;", Object::Boolean(true)),
            (r#""hello" == "hello";"#, Object::Boolean(true)),
            (r#""hello" == "hi";"#, Object::Boolean(false)),
            (r#""foo" + "bar";"#, Object::String("foobar".into())),
        ];

        for (expr_str, expected) in cases {
            let scanner = Scanner::new(expr_str);
            let mut parser = Parser::new(scanner.scan_tokens().map(|t| t.unwrap()));
            let ast = parser.parse();
            let Program::Declarations(decls) = ast;
            let mut env = Environment::default();
            for decl in decls {
                match decl {
                    Declaration::Statement(Stmt::Expr(expr)) => {
                        let result = eval::eval(&expr, &mut env);
                        assert_eq!(expected, result);
                    }
                    _ => panic!("test can only include expressions"),
                }
            }
        }
    }
}
