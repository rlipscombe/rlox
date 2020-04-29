#[macro_use]
extern crate lalrpop_util;
lalrpop_mod!(pub lox);

use codespan_reporting::diagnostic::{Diagnostic, Label};
use codespan_reporting::files::SimpleFiles;
use codespan_reporting::term::termcolor::{ColorChoice, StandardStream};

pub mod ast;
pub mod environment;
use environment::Environment;

mod test;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() <= 1 {
        eprintln!("{} file.lox", args[0]);
        return;
    }

    let path = &args[1];
    let source = std::fs::read_to_string(path).expect("read file");

    let mut environment = Environment::new();
    environment.push();
    match interpret_source(&source, &mut environment) {
        Ok(_) => {}
        Err(e) => report_error(&path, &source, e),
    };
}

fn interpret_source<'s>(
    source: &'s str,
    environment: &mut Environment,
) -> Result<(), Error<'s>> {
    let parser = lox::ProgramParser::new();
    let program = parser.parse(source).map_err(|e| Error::Parse(e))?;
    interpret_statements(program, environment)
}

fn interpret_statements<'s>(
    statements: Vec<ast::Stmt>,
    environment: &mut Environment,
) -> Result<(), Error<'s>> {
    for s in statements {
        interpret_statement(s, environment)?;
    }
    Ok(())
}

fn interpret_statement<'s>(
    statement: ast::Stmt,
    environment: &mut Environment,
) -> Result<(), Error<'s>> {
    use ast::Stmt::*;
    match statement {
        Empty => Ok(()),
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
        Block(statements) => {
            environment.push();
            let result = interpret_statements(statements, environment);
            environment.pop();
            result
        }
        If { cond, then, else_ } => {
            match evaluate(cond, environment) {
                Ok(Value::Boolean(true)) => {
                    interpret_statement(*then, environment)
                }
                Ok(Value::Boolean(false)) => {
                    interpret_statement(*else_, environment)
                }
                Ok(_) => {
                    Err(Error::Runtime(RuntimeError::TypeMismatch))
                }
                Err(e) => Err(e)
            }
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

pub fn evaluate<'s>(
    expr: ast::Expr,
    environment: &mut Environment,
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
    let mut files = SimpleFiles::new();
    let file_id = files.add(path, source);
    let diagnostic = match e {
        Error::Parse(ParseError::UnrecognizedToken {
            token: (start, _, end),
            expected,
        }) => {
            Diagnostic::error()
                .with_message("unrecognized token")
                .with_notes(expected)
                .with_labels(vec![Label::primary(file_id, start..end)])
        }
        Error::Parse(ParseError::InvalidToken { location: start }) => {
            Diagnostic::error()
                .with_message("invalid token")
                .with_labels(vec![Label::primary(file_id, start..start+1)])
        }
        Error::Runtime(RuntimeError::IdentifierNotFound { name, location }) => {
            Diagnostic::error()
                .with_message(format!("identifier '{}' not found", name))
                .with_labels(vec![Label::primary(file_id, location.start..location.end)])
        }
        Error::Assert { location } => {
            Diagnostic::error()
                .with_message("assertion failed")
                .with_labels(vec![Label::primary(file_id, location.start..location.end)])
        }
        error => Diagnostic::error().with_message(format!("{:?}", error)),
    };

    let writer = StandardStream::stderr(ColorChoice::Auto);
    let config = codespan_reporting::term::Config::default();
    let _ = codespan_reporting::term::emit(&mut writer.lock(), &config, &files, &diagnostic);
}
