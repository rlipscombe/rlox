#[macro_use]
extern crate lalrpop_util;
lalrpop_mod!(pub lox);

pub mod ast;

fn main() {
    let source = r#"
    print "Hello World!";
    print 45 + 65;
    "#;
    match interpret_source(source) {
        Ok(_) => {}
        Err(e) => {
            eprintln!("{:?}", e);
        }
    };
}

fn interpret_source(source: &str) -> Result<(), Error> {
    let parser = lox::ProgramParser::new();
    let program = parser.parse(source).map_err(|e| Error::Parse(e))?;
    interpret_statements(program)
}

fn interpret_statements<'s>(statements: Vec<Box<ast::Stmt>>) -> Result<(), Error<'s>> {
    for s in statements {
        interpret_statement(s)?;
    }
    Ok(())
}

fn interpret_statement<'s>(statement: Box<ast::Stmt>) -> Result<(), Error<'s>> {
    use ast::Stmt::*;
    match *statement {
        Expr(e) => {
            evaluate(e)?;
            Ok(())
        }
        Print(e) => {
            do_print(evaluate(e)?);
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

#[derive(PartialEq, Debug)]
enum Value {
    Nil,
    Number(f64),
    Boolean(bool),
    String(String),
}

fn do_add<'s>(lhs: Value, rhs: Value) -> Result<Value, Error<'s>> {
    match (lhs, rhs) {
        (Value::Number(l), Value::Number(r)) => Ok(Value::Number(l + r)),
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

fn evaluate<'s>(expr: Box<ast::Expr>) -> Result<Value, Error<'s>> {
    match *expr {
        ast::Expr::Nil => Ok(Value::Nil),
        ast::Expr::Number(n) => Ok(Value::Number(n)),
        ast::Expr::Boolean(b) => Ok(Value::Boolean(b)),
        ast::Expr::String(s) => Ok(Value::String(s)),
        ast::Expr::Unary(o, r) => match o {
            ast::UnaryOp::Invert => match evaluate(r)? {
                Value::Boolean(b) => Ok(Value::Boolean(!b)),
                _ => Err(Error::Runtime(RuntimeError::TypeMismatch)),
            },
            ast::UnaryOp::Negate => match evaluate(r)? {
                Value::Number(n) => Ok(Value::Number(-n)),
                _ => Err(Error::Runtime(RuntimeError::TypeMismatch)),
            },
        },
        ast::Expr::Binary(l, o, r) => match o {
            ast::BinaryOp::Add => do_add(evaluate(l)?, evaluate(r)?),
            ast::BinaryOp::Sub => do_sub(evaluate(l)?, evaluate(r)?),
            ast::BinaryOp::Mul => do_mul(evaluate(l)?, evaluate(r)?),
            ast::BinaryOp::Div => do_div(evaluate(l)?, evaluate(r)?),
            ast::BinaryOp::Mod => do_mod(evaluate(l)?, evaluate(r)?),
            ast::BinaryOp::Eq => do_eq(evaluate(l)?, evaluate(r)?),
            ast::BinaryOp::Ne => do_ne(evaluate(l)?, evaluate(r)?),
            ast::BinaryOp::Lt => do_lt(evaluate(l)?, evaluate(r)?),
            ast::BinaryOp::Le => do_le(evaluate(l)?, evaluate(r)?),
            ast::BinaryOp::Gt => do_gt(evaluate(l)?, evaluate(r)?),
            ast::BinaryOp::Ge => do_ge(evaluate(l)?, evaluate(r)?),
        },
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
    TypeMismatch
}

fn parse_string(source: &str) -> Result<Box<ast::Expr>, Error> {
    let parser = lox::ExprParser::new();
    parser.parse(source).map_err(|e| Error::Parse(e))
}

fn evaluate_string(source: &str) -> Result<Value, Error> {
    let result = parse_string(source);
    result.and_then(evaluate)
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
