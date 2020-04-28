#[macro_use]
extern crate lalrpop_util;

lalrpop_mod!(pub lox);

pub mod ast;
use crate::lox::Token;

fn main() {
    let path = "<stdin>";
    let source = "-1234 + 67 * 0.2 * 1.5 / 3";
/*    let parser = lox::ExprParser::new();
*/
}

#[test]
fn literal_number() {
    assert_eq!(Ok(123.0), evaluate_string("123"));
}

#[test]
fn paren_number() {
    assert_eq!(Ok(123.0), evaluate_string("(123)"));
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

fn evaluate(expr: Box<ast::Expr>) -> f64 {
    match *expr {
        ast::Expr::Number(n) => n,
        ast::Expr::Binary(l, o, r) => {
            match o {
                ast::BinaryOp::Add => { evaluate(l) + evaluate(r) }
                ast::BinaryOp::Sub => { evaluate(l) - evaluate(r) }
                ast::BinaryOp::Mul => { evaluate(l) * evaluate(r) }
                ast::BinaryOp::Div => { evaluate(l) / evaluate(r) }
            }
        }
    }
}

#[cfg(test)]
type Error<'s> = lalrpop_util::ParseError<usize, lox::Token<'s>, &'s str>;

#[cfg(test)]
fn evaluate_string(source: &str) -> Result<f64, Error> {
    let parser = lox::ExprParser::new();
    let result: Result<Box<ast::Expr>, Error> = parser.parse(source);
    result.map(evaluate)
}

#[test]
fn negate_number() {
    assert_eq!(Ok(-123.0), evaluate_string("-123"));
}

#[test]
fn simple_addition() {
    assert_eq!(Ok(1234.0 + 67.0), evaluate_string("1234 + 67"));
}

#[test]
fn repeated_addition() {
    assert_eq!(Ok(6.0), evaluate_string("1 + 2 + 3"));
}

#[test]
fn simple_subtraction() {
    assert_eq!(Ok(1200.0), evaluate_string("1234 - 34"));
}

#[test]
fn repeated_subtraction() {
    assert_eq!(Ok(4.0), evaluate_string("9 - 4 - 1"));
}

#[test]
fn mul_div() {
    assert_eq!(Ok(12.0), evaluate_string("9 * 4 / 3"));
}

#[test]
fn div_precedence() {
    assert_eq!(Ok(18.5), evaluate_string("17 - 5 / 2 + 4"));
}
