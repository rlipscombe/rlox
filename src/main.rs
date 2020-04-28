#[macro_use]
extern crate lalrpop_util;
lalrpop_mod!(pub lox);

pub mod ast;

fn main() {
    println!("{:?}", evaluate_string("-123"));
}

#[derive(PartialEq, Debug)]
enum Value {
    Nil,
    Number(f64),
    Boolean(bool),
}

fn do_add<'s>(lhs: Value, rhs: Value) -> Result<Value, Error<'s>> {
    match (lhs, rhs) {
        (Value::Number(l), Value::Number(r)) => Ok(Value::Number(l + r)),
        _ => Err(Error::Runtime)
    }
}

fn do_sub<'s>(lhs: Value, rhs: Value) -> Result<Value, Error<'s>> {
    match (lhs, rhs) {
        (Value::Number(l), Value::Number(r)) => Ok(Value::Number(l - r)),
        _ => Err(Error::Runtime)
    }
}

fn do_mul<'s>(lhs: Value, rhs: Value) -> Result<Value, Error<'s>> {
    match (lhs, rhs) {
        (Value::Number(l), Value::Number(r)) => Ok(Value::Number(l * r)),
        _ => Err(Error::Runtime)
    }
}

fn do_div<'s>(lhs: Value, rhs: Value) -> Result<Value, Error<'s>> {
    match (lhs, rhs) {
        (Value::Number(l), Value::Number(r)) => Ok(Value::Number(l / r)),
        _ => Err(Error::Runtime)
    }
}

fn do_mod<'s>(lhs: Value, rhs: Value) -> Result<Value, Error<'s>> {
    match (lhs, rhs) {
        (Value::Number(l), Value::Number(r)) => Ok(Value::Number(l % r)),
        _ => Err(Error::Runtime)
    }
}

fn do_eq<'s>(lhs: Value, rhs: Value) -> Result<Value, Error<'s>> {
    match (lhs, rhs) {
        (Value::Number(l), Value::Number(r)) => Ok(Value::Boolean(l == r)),
        _ => Err(Error::Runtime)
    }
}

fn do_ne<'s>(lhs: Value, rhs: Value) -> Result<Value, Error<'s>> {
    match (lhs, rhs) {
        (Value::Number(l), Value::Number(r)) => Ok(Value::Boolean(l != r)),
        _ => Err(Error::Runtime)
    }
}

fn do_lt<'s>(lhs: Value, rhs: Value) -> Result<Value, Error<'s>> {
    match (lhs, rhs) {
        (Value::Number(l), Value::Number(r)) => Ok(Value::Boolean(l < r)),
        _ => Err(Error::Runtime)
    }
}

fn do_le<'s>(lhs: Value, rhs: Value) -> Result<Value, Error<'s>> {
    match (lhs, rhs) {
        (Value::Number(l), Value::Number(r)) => Ok(Value::Boolean(l <= r)),
        _ => Err(Error::Runtime)
    }
}

fn do_gt<'s>(lhs: Value, rhs: Value) -> Result<Value, Error<'s>> {
    match (lhs, rhs) {
        (Value::Number(l), Value::Number(r)) => Ok(Value::Boolean(l > r)),
        _ => Err(Error::Runtime)
    }
}

fn do_ge<'s>(lhs: Value, rhs: Value) -> Result<Value, Error<'s>> {
    match (lhs, rhs) {
        (Value::Number(l), Value::Number(r)) => Ok(Value::Boolean(l >= r)),
        _ => Err(Error::Runtime)
    }
}

fn evaluate<'s>(expr: Box<ast::Expr>) -> Result<Value, Error<'s>> {
    match *expr {
        ast::Expr::Nil => Ok(Value::Nil),
        ast::Expr::Number(n) => Ok(Value::Number(n)),
        ast::Expr::Boolean(b) => Ok(Value::Boolean(b)),
        ast::Expr::Binary(l, o, r) => {
            match o {
                ast::BinaryOp::Add => { do_add(evaluate(l)?, evaluate(r)?) }
                ast::BinaryOp::Sub => { do_sub(evaluate(l)?, evaluate(r)?) }
                ast::BinaryOp::Mul => { do_mul(evaluate(l)?, evaluate(r)?) }
                ast::BinaryOp::Div => { do_div(evaluate(l)?, evaluate(r)?) }
                ast::BinaryOp::Mod => { do_mod(evaluate(l)?, evaluate(r)?) }
                ast::BinaryOp::Eq => { do_eq(evaluate(l)?, evaluate(r)?) }
                ast::BinaryOp::Ne => { do_ne(evaluate(l)?, evaluate(r)?) }
                ast::BinaryOp::Lt => { do_lt(evaluate(l)?, evaluate(r)?) }
                ast::BinaryOp::Le => { do_le(evaluate(l)?, evaluate(r)?) }
                ast::BinaryOp::Gt => { do_gt(evaluate(l)?, evaluate(r)?) }
                ast::BinaryOp::Ge => { do_ge(evaluate(l)?, evaluate(r)?) }
            }
        }
    }
}

type ParseError<'s> = lalrpop_util::ParseError<usize, lox::Token<'s>, &'s str>;

#[derive(Debug, PartialEq)]
enum Error<'s> {
    Parse(ParseError<'s>),
    Runtime
}


fn parse_string(source: &str) -> Result<Box<ast::Expr>, Error> {
    let parser = lox::ExprParser::new();
    parser.parse(source).map_err(|e| { Error::Parse(e) })
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
    assert_eq!(Ok(Value::Number(1234.0 + 67.0)), evaluate_string("1234 + 67"));
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
