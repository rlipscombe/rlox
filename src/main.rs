#[macro_use]
extern crate lalrpop_util;
lalrpop_mod!(pub lox);

use std::collections::HashMap;

pub mod ast;

fn main() {
    let path = "<verbatim>";
    let source = r#"
    var greeting = "Hello";
    var recipient = "World!";
    print greeting + " " + recipient;
    var PI = 3.1415;
    var r = 2;
    var area = PI * r * r;
    print area;
    // This next line has an error.
    var volume = PI * r * r * height;
    print volume;
    "#;
    let mut environment = Environment::new();
    match interpret_source(source, &mut environment) {
        Ok(_) => {}
        Err(e) => match e {
            Error::Parse(ParseError::UnrecognizedToken{token:(start,_, end), expected}) => {
                let location = (start, end);
                if let Some((line_num, line, offset)) = get_source_at_location(source, location) {
                    eprintln!(
                        "{}: unrecognized token at line {}:{}:{}; expected one of {}",
                        path, line_num, offset.0, offset.1, expected.join(", ")
                    );
                    eprintln!("{}", line);
                    eprintln!("{:>offset$}{:^>length$}", "", "^", offset = offset.0 - 1, length = offset.1 - offset.0 + 1);
                } else {
                    eprintln!("{}: unrecognized token; expected one of {}", path, expected.join(", "));
                }
            }
            Error::Runtime(RuntimeError::IdentifierNotFound { name, location }) => {
                if let Some((line_num, line, offset)) = get_source_at_location(source, location) {
                    eprintln!(
                        "{}: identifier '{}' not found at line {}:{}:{}",
                        path, name, line_num, offset.0, offset.1
                    );
                    eprintln!("{}", line);
                    eprintln!("{:>offset$}{:^>length$}", "", "^", offset = offset.0 - 1, length = offset.1 - offset.0 + 1);
                } else {
                    eprintln!("{}: identifier '{}' not found", path, name);
                }
            }
            _ => {
                eprintln!("{:?}", e);
            }
        },
    };
}

fn get_source_at_location(
    source: &str,
    location: (usize, usize),
) -> Option<(usize, &str, (usize, usize))> {
    let mut lines: Vec<(usize, &str)> = Vec::new();
    let mut offset = 0;
    for s in source.split('\n') {
        lines.push((offset, s));
        offset += s.len() + 1;
    }

    let mut line = 1;
    for (x, s) in lines {
        if x + s.len() > location.0 {
            return Some((line, s, (location.0 - x + 1, location.1 - x)));
        }
        line += 1;
    }
    None
}

fn interpret_source<'s>(
    source: &'s str,
    environment: &'s mut Environment,
) -> Result<(), Error<'s>> {
    let parser = lox::ProgramParser::new();
    let program = parser.parse(source).map_err(|e| Error::Parse(e))?;
    interpret_statements(program, environment)
}

fn interpret_statements<'s>(
    statements: Vec<Box<ast::Stmt>>,
    environment: &'s mut Environment,
) -> Result<(), Error<'s>> {
    for s in statements {
        interpret_statement(s, environment)?;
    }
    Ok(())
}

struct Environment {
    values: HashMap<String, Value>,
}

impl Environment {
    fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }
    fn define(&mut self, name: &str, value: Value) {
        self.values.insert(name.to_string(), value);
    }
    fn get(&self, name: &str) -> Option<Value> {
        self.values.get(name).and_then(|v| Some(v.clone()))
    }
}

fn interpret_statement<'s, 'e>(
    statement: Box<ast::Stmt>,
    environment: &'e mut Environment,
) -> Result<(), Error<'s>> {
    use ast::Stmt::*;
    match *statement {
        Expr(e) => {
            evaluate(e, environment)?;
            Ok(())
        }
        Print(e) => {
            do_print(evaluate(e, environment)?);
            Ok(())
        }
        VarDecl(i, e) => {
            environment.define(&i, evaluate(e, environment)?);
            Ok(())
        }
    }
}

fn do_print(e: Value) {
    use Value::*;
    match e {
        Nil => println!("<nil>"),
        Number(n) => println!("{}", n),
        Boolean(b) => println!("{}", b),
        String(s) => println!("{}", s),
    }
}

#[derive(PartialEq, Clone, Debug)]
enum Value {
    Nil,
    Number(f64),
    Boolean(bool),
    String(String),
}

fn do_add<'s>(lhs: Value, rhs: Value) -> Result<Value, Error<'s>> {
    match (lhs, rhs) {
        (Value::Number(l), Value::Number(r)) => Ok(Value::Number(l + r)),
        (Value::String(l), Value::String(r)) => Ok(Value::String(l + &r)),
        _ => Err(Error::Runtime(RuntimeError::TypeMismatch)),
    }
}

fn do_sub<'s>(lhs: Value, rhs: Value) -> Result<Value, Error<'s>> {
    match (lhs, rhs) {
        (Value::Number(l), Value::Number(r)) => Ok(Value::Number(l - r)),
        _ => Err(Error::Runtime(RuntimeError::TypeMismatch)),
    }
}

fn do_mul<'s>(lhs: Value, rhs: Value) -> Result<Value, Error<'s>> {
    match (lhs, rhs) {
        (Value::Number(l), Value::Number(r)) => Ok(Value::Number(l * r)),
        _ => Err(Error::Runtime(RuntimeError::TypeMismatch)),
    }
}

fn do_div<'s>(lhs: Value, rhs: Value) -> Result<Value, Error<'s>> {
    match (lhs, rhs) {
        (Value::Number(l), Value::Number(r)) => Ok(Value::Number(l / r)),
        _ => Err(Error::Runtime(RuntimeError::TypeMismatch)),
    }
}

fn do_mod<'s>(lhs: Value, rhs: Value) -> Result<Value, Error<'s>> {
    match (lhs, rhs) {
        (Value::Number(l), Value::Number(r)) => Ok(Value::Number(l % r)),
        _ => Err(Error::Runtime(RuntimeError::TypeMismatch)),
    }
}

fn do_eq<'s>(lhs: Value, rhs: Value) -> Result<Value, Error<'s>> {
    match (lhs, rhs) {
        (Value::Number(l), Value::Number(r)) => Ok(Value::Boolean(l == r)),
        _ => Err(Error::Runtime(RuntimeError::TypeMismatch)),
    }
}

fn do_ne<'s>(lhs: Value, rhs: Value) -> Result<Value, Error<'s>> {
    match (lhs, rhs) {
        (Value::Number(l), Value::Number(r)) => Ok(Value::Boolean(l != r)),
        _ => Err(Error::Runtime(RuntimeError::TypeMismatch)),
    }
}

fn do_lt<'s>(lhs: Value, rhs: Value) -> Result<Value, Error<'s>> {
    match (lhs, rhs) {
        (Value::Number(l), Value::Number(r)) => Ok(Value::Boolean(l < r)),
        _ => Err(Error::Runtime(RuntimeError::TypeMismatch)),
    }
}

fn do_le<'s>(lhs: Value, rhs: Value) -> Result<Value, Error<'s>> {
    match (lhs, rhs) {
        (Value::Number(l), Value::Number(r)) => Ok(Value::Boolean(l <= r)),
        _ => Err(Error::Runtime(RuntimeError::TypeMismatch)),
    }
}

fn do_gt<'s>(lhs: Value, rhs: Value) -> Result<Value, Error<'s>> {
    match (lhs, rhs) {
        (Value::Number(l), Value::Number(r)) => Ok(Value::Boolean(l > r)),
        _ => Err(Error::Runtime(RuntimeError::TypeMismatch)),
    }
}

fn do_ge<'s>(lhs: Value, rhs: Value) -> Result<Value, Error<'s>> {
    match (lhs, rhs) {
        (Value::Number(l), Value::Number(r)) => Ok(Value::Boolean(l >= r)),
        _ => Err(Error::Runtime(RuntimeError::TypeMismatch)),
    }
}

fn evaluate<'s, 'e>(
    expr: Box<ast::Expr>,
    environment: &'e Environment,
) -> Result<Value, Error<'s>> {
    match *expr {
        ast::Expr::Nil => Ok(Value::Nil),
        ast::Expr::Number(n) => Ok(Value::Number(n)),
        ast::Expr::Boolean(b) => Ok(Value::Boolean(b)),
        ast::Expr::String(s) => Ok(Value::String(s)),
        ast::Expr::Unary(o, r) => match o {
            ast::UnaryOp::Invert => match evaluate(r, environment)? {
                Value::Boolean(b) => Ok(Value::Boolean(!b)),
                _ => Err(Error::Runtime(RuntimeError::TypeMismatch)),
            },
            ast::UnaryOp::Negate => match evaluate(r, environment)? {
                Value::Number(n) => Ok(Value::Number(-n)),
                _ => Err(Error::Runtime(RuntimeError::TypeMismatch)),
            },
        },
        ast::Expr::Binary(l, o, r) => match o {
            ast::BinaryOp::Add => do_add(evaluate(l, environment)?, evaluate(r, environment)?),
            ast::BinaryOp::Sub => do_sub(evaluate(l, environment)?, evaluate(r, environment)?),
            ast::BinaryOp::Mul => do_mul(evaluate(l, environment)?, evaluate(r, environment)?),
            ast::BinaryOp::Div => do_div(evaluate(l, environment)?, evaluate(r, environment)?),
            ast::BinaryOp::Mod => do_mod(evaluate(l, environment)?, evaluate(r, environment)?),
            ast::BinaryOp::Eq => do_eq(evaluate(l, environment)?, evaluate(r, environment)?),
            ast::BinaryOp::Ne => do_ne(evaluate(l, environment)?, evaluate(r, environment)?),
            ast::BinaryOp::Lt => do_lt(evaluate(l, environment)?, evaluate(r, environment)?),
            ast::BinaryOp::Le => do_le(evaluate(l, environment)?, evaluate(r, environment)?),
            ast::BinaryOp::Gt => do_gt(evaluate(l, environment)?, evaluate(r, environment)?),
            ast::BinaryOp::Ge => do_ge(evaluate(l, environment)?, evaluate(r, environment)?),
        },
        ast::Expr::Var { name, location } => environment
            .get(&name)
            .ok_or_else(|| Error::Runtime(RuntimeError::IdentifierNotFound { name, location })),
    }
}

type ParseError<'s> = lalrpop_util::ParseError<usize, lox::Token<'s>, &'s str>;

#[derive(Debug, PartialEq)]
enum Error<'s> {
    Parse(ParseError<'s>),
    Runtime(RuntimeError),
}

#[derive(Debug, PartialEq)]
enum RuntimeError {
    TypeMismatch,
    IdentifierNotFound {
        name: String,
        location: (usize, usize),
    },
}

fn parse_string(source: &str) -> Result<Box<ast::Expr>, Error> {
    let parser = lox::ExprParser::new();
    parser.parse(source).map_err(|e| Error::Parse(e))
}

fn evaluate_string(source: &str) -> Result<Value, Error> {
    let result = parse_string(source);
    let environment = Environment::new();
    result.and_then(|expr| evaluate(expr, &environment))
}

#[test]
fn literal_number() {
    assert_eq!(Ok(Value::Number(123.0)), evaluate_string("123"));
}

#[test]
fn paren_number() {
    assert_eq!(Ok(Value::Number(123.0)), evaluate_string("(123)"));
}

#[test]
fn paren_imbalanced() {
    let parser = lox::ExprParser::new();
    assert!(parser.parse("(123").is_err());
}

#[test]
fn paren_empty() {
    let parser = lox::ExprParser::new();
    assert!(parser.parse("()").is_err());
}

#[test]
fn negate_number() {
    assert_eq!(Ok(Value::Number(-123.0)), evaluate_string("-123"));
}

#[test]
fn simple_addition() {
    assert_eq!(
        Ok(Value::Number(1234.0 + 67.0)),
        evaluate_string("1234 + 67")
    );
}

#[test]
fn repeated_addition() {
    assert_eq!(Ok(Value::Number(6.0)), evaluate_string("1 + 2 + 3"));
}

#[test]
fn simple_subtraction() {
    assert_eq!(Ok(Value::Number(1200.0)), evaluate_string("1234 - 34"));
}

#[test]
fn repeated_subtraction() {
    assert_eq!(Ok(Value::Number(4.0)), evaluate_string("9 - 4 - 1"));
}

#[test]
fn mul_div() {
    assert_eq!(Ok(Value::Number(12.0)), evaluate_string("9 * 4 / 3"));
}

#[test]
fn div_precedence() {
    assert_eq!(Ok(Value::Number(18.5)), evaluate_string("17 - 5 / 2 + 4"));
}

#[test]
fn mul_nil() {
    assert!(evaluate_string("nil * nil").is_err());
}

#[test]
fn cmp_lt() {
    assert_eq!(evaluate_string("1 < 2"), Ok(Value::Boolean(true)));
    assert_eq!(evaluate_string("2 < 1"), Ok(Value::Boolean(false)));
}

#[test]
fn cmp_le() {
    assert_eq!(evaluate_string("1 <= 2"), Ok(Value::Boolean(true)));
    assert_eq!(evaluate_string("2 <= 2"), Ok(Value::Boolean(true)));
    assert_eq!(evaluate_string("2 <= 1"), Ok(Value::Boolean(false)));
}

#[test]
fn cmp_gt() {
    assert_eq!(evaluate_string("2 > 1"), Ok(Value::Boolean(true)));
    assert_eq!(evaluate_string("1 > 2"), Ok(Value::Boolean(false)));
}

#[test]
fn cmp_ge() {
    assert_eq!(evaluate_string("2 >= 1"), Ok(Value::Boolean(true)));
    assert_eq!(evaluate_string("2 >= 2"), Ok(Value::Boolean(true)));
    assert_eq!(evaluate_string("1 >= 2"), Ok(Value::Boolean(false)));
}

#[test]
fn cmp_eq() {
    assert_eq!(evaluate_string("2 != 1"), Ok(Value::Boolean(true)));
    assert_eq!(evaluate_string("2 == 2"), Ok(Value::Boolean(true)));
    assert_eq!(evaluate_string("(1 + 1) == 2"), Ok(Value::Boolean(true)));
}

#[test]
fn simple_bool() {
    assert_eq!(evaluate_string("true"), Ok(Value::Boolean(true)));
    assert_eq!(evaluate_string("false"), Ok(Value::Boolean(false)));
}

#[test]
fn negate_bool() {
    assert_eq!(evaluate_string("!true"), Ok(Value::Boolean(false)));
    assert_eq!(evaluate_string("!false"), Ok(Value::Boolean(true)));
}

#[test]
fn double_negate_bool() {
    assert_eq!(evaluate_string("!!true"), Ok(Value::Boolean(true)));
    assert_eq!(evaluate_string("!!(1 == 2)"), Ok(Value::Boolean(false)));
}

#[test]
fn string_literal() {
    assert_eq!(
        evaluate_string(r#""Hello World!""#),
        Ok(Value::String("Hello World!".to_string()))
    );
}

#[test]
fn empty_string_literal() {
    assert_eq!(evaluate_string(r#""""#), Ok(Value::String("".to_string())));
}

#[test]
fn print_hello() {
    use ast::*;
    let parser = lox::StatementParser::new();
    let parsed = parser.parse(r#"print "Hello World!";"#);
    assert_eq!(
        parsed,
        Ok(Box::new(Stmt::Print(Box::new(Expr::String(
            "Hello World!".to_string()
        )))))
    );
}
