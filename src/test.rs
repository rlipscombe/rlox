#[cfg(test)]
use crate::*;

#[cfg(test)]
fn parse_string(source: &str) -> Result<ast::Expr, Error> {
    let parser = lox::ExprParser::new();
    parser.parse(source).map_err(|e| Error::Parse(e))
}

#[cfg(test)]
fn evaluate_string(source: &str) -> Result<Value, Error> {
    let result = parse_string(source);
    let mut environment = Environment::new();
    result.and_then(|expr| evaluate(&expr, &mut environment))
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
        Ok(Stmt::Print(Expr::String {
            value: "Hello World!".to_string(),
            location: Location { start: 6, end: 20 }
        }))
    );
}
