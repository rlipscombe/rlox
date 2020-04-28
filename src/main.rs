#[macro_use]
extern crate lalrpop_util;
lalrpop_mod!(pub lox);

pub mod ast;
pub mod environment;
use environment::Environment;

mod test;

fn main() {
    let path = "<verbatim>";
    let source = r#"
    // String catenation
    var greeting = "Hello";
    var recipient = "World!";
    print greeting + " " + recipient;

    // Area of a circle
    var PI = 3.1415;    // vardecl w/ initializer
    var r;  // test vardecl w/o initializer
    r = 3;  // test assignment
    var area = PI * r * r;
    print area;

    // Assignment should be right-associative
    var x;
    {
        // BLOCK:
        var y;
        x = y = 42;
    }
    print x;

    // Scoping
    var a = "global a";
    var b = "global b";
    var c = "global c";
    {
      var a = "outer a";
      var b = "outer b";
      {
        var a = "inner a";
        assert a == "inner a";
        assert b == "outer b";
        assert c == "global c";
      }
      assert a == "outer a";
      assert b == "outer b";
      assert c == "global c";
    }
    assert a == "global a";
    assert b == "global b";
    assert c == "global c";

    // This next line has an error: height is undefined.
    var volume = PI * r * r * height;
    print volume;
    "#;
    let mut environment = Environment::new();
    match interpret_source(source, &mut environment) {
        Ok(_) => {}
        Err(e) => report_error(path, source, e),
    };
}

fn interpret_source<'s>(
    source: &'s str,
    environment: &'s mut Environment,
) -> Result<(), Error<'s>> {
    let parser = lox::ProgramParser::new();
    let program = parser.parse(source).map_err(|e| Error::Parse(e))?;
    interpret_statements(program, environment)
}

fn interpret_statements<'s, 'e>(
    statements: Vec<ast::Stmt>,
    environment: &'e mut Environment,
) -> Result<(), Error<'s>> {
    for s in statements {
        interpret_statement(s, environment)?;
    }
    Ok(())
}

fn interpret_statement<'s, 'e>(
    statement: ast::Stmt,
    environment: &'e mut Environment,
) -> Result<(), Error<'s>> {
    use ast::Stmt::*;
    match statement {
        Expr(e) => {
            evaluate(*e, environment)?;
            Ok(())
        }
        Print(e) => {
            do_print(evaluate(*e, environment)?);
            Ok(())
        }
        Assert { expr, location } => match evaluate(*expr, environment)? {
            Value::Nil => Err(Error::Assert { location }),
            Value::Boolean(false) => Err(Error::Assert { location }),
            _ => Ok(()),
        },
        VarDecl(i, e) => {
            let value = evaluate(*e, environment)?;
            environment.define(&i, value);
            Ok(())
        }
        Block(statements) => interpret_statements(statements, environment),
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
pub enum Value {
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
        (Value::String(l), Value::String(r)) => Ok(Value::Boolean(l == r)),
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

pub fn evaluate<'s, 'e>(
    expr: ast::Expr,
    environment: &'e mut Environment,
) -> Result<Value, Error<'s>> {
    match expr {
        ast::Expr::Nil => Ok(Value::Nil),
        ast::Expr::Number(n) => Ok(Value::Number(n)),
        ast::Expr::Boolean(b) => Ok(Value::Boolean(b)),
        ast::Expr::String(s) => Ok(Value::String(s)),
        ast::Expr::Unary(o, r) => match o {
            ast::UnaryOp::Invert => match evaluate(*r, environment)? {
                Value::Boolean(b) => Ok(Value::Boolean(!b)),
                _ => Err(Error::Runtime(RuntimeError::TypeMismatch)),
            },
            ast::UnaryOp::Negate => match evaluate(*r, environment)? {
                Value::Number(n) => Ok(Value::Number(-n)),
                _ => Err(Error::Runtime(RuntimeError::TypeMismatch)),
            },
        },
        ast::Expr::Binary(l, o, r) => match o {
            ast::BinaryOp::Add => do_add(evaluate(*l, environment)?, evaluate(*r, environment)?),
            ast::BinaryOp::Sub => do_sub(evaluate(*l, environment)?, evaluate(*r, environment)?),
            ast::BinaryOp::Mul => do_mul(evaluate(*l, environment)?, evaluate(*r, environment)?),
            ast::BinaryOp::Div => do_div(evaluate(*l, environment)?, evaluate(*r, environment)?),
            ast::BinaryOp::Mod => do_mod(evaluate(*l, environment)?, evaluate(*r, environment)?),
            ast::BinaryOp::Eq => do_eq(evaluate(*l, environment)?, evaluate(*r, environment)?),
            ast::BinaryOp::Ne => do_ne(evaluate(*l, environment)?, evaluate(*r, environment)?),
            ast::BinaryOp::Lt => do_lt(evaluate(*l, environment)?, evaluate(*r, environment)?),
            ast::BinaryOp::Le => do_le(evaluate(*l, environment)?, evaluate(*r, environment)?),
            ast::BinaryOp::Gt => do_gt(evaluate(*l, environment)?, evaluate(*r, environment)?),
            ast::BinaryOp::Ge => do_ge(evaluate(*l, environment)?, evaluate(*r, environment)?),
        },
        ast::Expr::Var { name, location } => environment
            .get(&name)
            .ok_or_else(|| Error::Runtime(RuntimeError::IdentifierNotFound { name, location })),
        ast::Expr::Assignment {
            name,
            rhs,
            location,
        } => {
            let value = evaluate(*rhs, environment)?;
            environment.assign(&name, value).or(Err(Error::Runtime(
                RuntimeError::IdentifierNotFound { name, location },
            )))
        }
    }
}

type ParseError<'s> = lalrpop_util::ParseError<usize, lox::Token<'s>, &'s str>;

#[derive(Debug, PartialEq)]
pub enum Error<'s> {
    Parse(ParseError<'s>),
    Runtime(RuntimeError),
    Assert { location: ast::Location },
}

#[derive(Debug, PartialEq)]
pub enum RuntimeError {
    TypeMismatch,
    IdentifierNotFound {
        name: String,
        location: ast::Location,
    },
}

fn report_error(path: &str, source: &str, e: Error) {
    match e {
        Error::Parse(ParseError::UnrecognizedToken {
            token: (start, _, end),
            expected,
        }) => {
            let location = ast::Location { start, end };
            let report = get_source_at_location(source, location);
            eprintln!(
                "error: unrecognized token; expected one of {}",
                expected.join(", ")
            );
            eprintln!("{}:{}:{}", path, report.line, report.start);
            report_offender(report);
        }
        Error::Parse(ParseError::InvalidToken { location: start }) => {
            let location = ast::Location {
                start,
                end: start + 1,
            };
            let report = get_source_at_location(source, location);
            eprintln!("error: invalid token");
            eprintln!("{}:{}:{}", path, report.line, report.start);
            report_offender(report);
        }
        Error::Runtime(RuntimeError::IdentifierNotFound { name, location }) => {
            let report = get_source_at_location(source, location);
            eprintln!("error: identifier '{}' not found", name);
            eprintln!("{}:{}:{}", path, report.line, report.start);
            report_offender(report);
        }
        Error::Assert{ location } => {
            let report = get_source_at_location(source, location);
            eprintln!("assertion failed");
            eprintln!("{}:{}:{}", path, report.line, report.start);
            report_offender(report);
        }
        _ => {
            eprintln!("{:?}", e);
        }
    }
}

fn report_offender(report: ReportLocation) {
    eprintln!("{}", report.text);
    eprintln!(
        "{:>offset$}{:^>length$}",
        "",
        "^",
        offset = report.start - 1,
        length = report.end - report.start + 1
    );
}

struct ReportLocation<'a> {
    line: usize,
    text: &'a str,
    start: usize,
    end: usize,
}

fn get_source_at_location(source: &str, location: ast::Location) -> ReportLocation {
    let mut lines: Vec<(usize, &str)> = Vec::new();
    let mut offset = 0;
    for s in source.split('\n') {
        lines.push((offset, s));
        offset += s.len() + 1;
    }

    let mut line = 1;
    for (x, s) in lines {
        if x + s.len() > location.start {
            return ReportLocation {
                line: line,
                text: s,
                start: location.start - x + 1,
                end: location.end - x,
            };
        }
        line += 1;
    }

    panic!();
}
