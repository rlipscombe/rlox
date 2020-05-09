#[macro_use]
extern crate lalrpop_util;
lalrpop_mod!(pub lox);

use std::time::{SystemTime, UNIX_EPOCH};

use codespan_reporting::diagnostic::{Diagnostic, Label};
use codespan_reporting::files::SimpleFiles;
use codespan_reporting::term::termcolor::{ColorChoice, StandardStream};

pub mod ast;
use ast::Locatable;

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
    let clock = Value::NativeFunction {
        name: "clock".to_string(),
        arity: 0,
        fun: |_argv| {
            let now = SystemTime::now();
            Value::Number(now.duration_since(UNIX_EPOCH).unwrap().as_millis() as f64)
        },
    };
    environment.define("clock", clock);
    match interpret_source(&source, &mut environment) {
        Ok(_) => {}
        Err(e) => report_error(&path, &source, e),
    };
}

fn interpret_source<'s>(source: &'s str, environment: &mut Environment) -> Result<(), Error<'s>> {
    let parser = lox::ProgramParser::new();
    let program = parser.parse(source).map_err(|e| Error::Parse(e))?;
    interpret_statements(&program, environment)
}

fn interpret_statements<'s>(
    statements: &Vec<ast::Stmt>,
    environment: &mut Environment,
) -> Result<(), Error<'s>> {
    for s in statements {
        interpret_statement(&s, environment)?;
    }
    Ok(())
}

fn interpret_statement<'s>(
    statement: &ast::Stmt,
    environment: &mut Environment,
) -> Result<(), Error<'s>> {
    use ast::Stmt::*;
    match statement {
        Empty => Ok(()),
        Expr(e) => {
            evaluate(&e, environment)?;
            Ok(())
        }
        Print(e) => {
            do_print(evaluate(&e, environment)?);
            Ok(())
        }
        Assert { expr, location } => match evaluate(&expr, environment)? {
            Value::Nil => Err(Error::Assert {
                location: *location,
            }),
            Value::Boolean(false) => Err(Error::Assert {
                location: *location,
            }),
            _ => Ok(()),
        },
        VarDecl(i, e) => {
            let value = evaluate(&e, environment)?;
            environment.define(&i, value);
            Ok(())
        }
        FunDecl {
            name, params, body, ..
        } => {
            let closure = environment.clone();
            let callable = Value::LoxFunction {
                name: name.to_string(),
                closure: closure,
                params: params.to_vec(),
                body: body.clone(),
            };
            environment.define(name, callable);
            Ok(())
        }
        Block(statements) => {
            let result = interpret_statements(statements, environment);
            result
        }
        If { cond, then, else_ } => match evaluate(&cond, environment) {
            Ok(Value::Boolean(true)) => interpret_statement(then, environment),
            Ok(Value::Boolean(false)) => interpret_statement(else_, environment),
            Ok(_) => Err(Error::Runtime(RuntimeError::TypeMismatch)),
            Err(e) => Err(e),
        },
        While { cond, body } => loop {
            match evaluate(&cond, environment) {
                Ok(Value::Boolean(true)) => {
                    interpret_statement(&body, environment)?;
                }
                Ok(Value::Boolean(false)) => {
                    return Ok(());
                }
                Ok(_) => {
                    return Err(Error::Runtime(RuntimeError::TypeMismatch));
                }
                Err(e) => return Err(e),
            }
        },
    }
}

fn do_print(e: Value) {
    use Value::*;
    match e {
        Nil => println!("<nil>"),
        Number(n) => println!("{}", n),
        Boolean(b) => println!("{}", b),
        String(s) => println!("{}", s),
        NativeFunction { name, .. } => println!("<fun {} (native)>", name),
        LoxFunction { name, .. } => println!("<fun {} (lox)>", name),
    }
}

#[derive(PartialEq, Clone, Debug)]
pub enum Value {
    Nil,
    Number(f64),
    Boolean(bool),
    String(String),
    NativeFunction {
        name: String,
        arity: usize,
        // TODO: Result<Value, RuntimeError>, but that needs lifetime
        // shenanigans.
        fun: fn(Vec<Value>) -> Value,
    },
    // TODO: implement "return"
    LoxFunction {
        name: String,
        closure: Environment,
        params: Vec<String>,
        body: Box<ast::Stmt>,
    },
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

pub fn evaluate<'s>(expr: &ast::Expr, environment: &mut Environment) -> Result<Value, Error<'s>> {
    match expr {
        ast::Expr::Nil { .. } => Ok(Value::Nil),
        ast::Expr::Number { value, .. } => Ok(Value::Number(*value)),
        ast::Expr::Boolean { value, .. } => Ok(Value::Boolean(*value)),
        ast::Expr::String { value, .. } => Ok(Value::String(value.into())),
        ast::Expr::Unary { op, right, .. } => match op {
            ast::UnaryOp::Invert => match evaluate(&right, environment)? {
                Value::Boolean(b) => Ok(Value::Boolean(!b)),
                _ => Err(Error::Runtime(RuntimeError::TypeMismatch)),
            },
            ast::UnaryOp::Negate => match evaluate(&right, environment)? {
                Value::Number(n) => Ok(Value::Number(-n)),
                _ => Err(Error::Runtime(RuntimeError::TypeMismatch)),
            },
        },
        ast::Expr::Binary {
            left, op, right, ..
        } => match op {
            ast::BinaryOp::Add => do_add(
                evaluate(&left, environment)?,
                evaluate(&right, environment)?,
            ),
            ast::BinaryOp::Sub => do_sub(
                evaluate(&left, environment)?,
                evaluate(&right, environment)?,
            ),
            ast::BinaryOp::Mul => do_mul(
                evaluate(&left, environment)?,
                evaluate(&right, environment)?,
            ),
            ast::BinaryOp::Div => do_div(
                evaluate(&left, environment)?,
                evaluate(&right, environment)?,
            ),
            ast::BinaryOp::Mod => do_mod(
                evaluate(&left, environment)?,
                evaluate(&right, environment)?,
            ),
            ast::BinaryOp::Eq => do_eq(
                evaluate(&left, environment)?,
                evaluate(&right, environment)?,
            ),
            ast::BinaryOp::Ne => do_ne(
                evaluate(&left, environment)?,
                evaluate(&right, environment)?,
            ),
            ast::BinaryOp::Lt => do_lt(
                evaluate(&left, environment)?,
                evaluate(&right, environment)?,
            ),
            ast::BinaryOp::Le => do_le(
                evaluate(&left, environment)?,
                evaluate(&right, environment)?,
            ),
            ast::BinaryOp::Gt => do_gt(
                evaluate(&left, environment)?,
                evaluate(&right, environment)?,
            ),
            ast::BinaryOp::Ge => do_ge(
                evaluate(&left, environment)?,
                evaluate(&right, environment)?,
            ),
        },
        ast::Expr::Var { name, .. } => environment.get(&name).ok_or_else(|| {
            Error::Runtime(RuntimeError::IdentifierNotFound {
                name: name.into(),
                location: expr.location(),
            })
        }),
        ast::Expr::Assignment { name, rhs, .. } => {
            let value = evaluate(&rhs, environment)?;
            environment.assign(&name, value).or(Err(Error::Runtime(
                RuntimeError::IdentifierNotFound {
                    name: name.into(),
                    location: expr.location(),
                },
            )))
        }
        ast::Expr::Call { callee, args, .. } => do_call(&callee, args, environment),
    }
}

fn do_call<'s>(
    callee: &ast::Expr,
    args: &Vec<ast::Expr>,
    environment: &mut Environment,
) -> Result<Value, Error<'s>> {
    // The Java reference implementation of Lox evaluates the callee,
    // then the arguments, and _then_ checks that the callee is actually
    // callable.
    let callable = evaluate(callee, environment)?;
    let mut argv = Vec::with_capacity(args.len());
    for a in args {
        argv.push(evaluate(a, environment)?);
    }
    match callable {
        Value::NativeFunction { fun, arity, .. } => {
            if argv.len() != arity {
                return Err(Error::Runtime(RuntimeError::ArityMismatch {
                    expected: arity,
                    actual: argv.len(),
                    location: callee.location(),
                }));
            }
            let r = fun(argv);
            Ok(r)
        }
        Value::LoxFunction {
            closure,
            params,
            body,
            ..
        } => {
            if argv.len() != params.len() {
                return Err(Error::Runtime(RuntimeError::ArityMismatch {
                    expected: params.len(),
                    actual: argv.len(),
                    location: callee.location(),
                }));
            }
            let mut environment = Environment::with_enclosing(&closure);
            for (p, v) in params.iter().zip(argv.iter()) {
                environment.define(p, v.clone());
            }
            interpret_statement(&body, &mut environment)?;
            Ok(Value::Nil)
        }
        _ => {
            let location = callee.location();
            Err(Error::Runtime(RuntimeError::NotCallable {
                location: location,
            }))
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
    NotCallable {
        location: ast::Location,
    },
    ArityMismatch {
        expected: usize,
        actual: usize,
        location: ast::Location,
    },
}

fn report_error(path: &str, source: &str, e: Error) {
    let mut files = SimpleFiles::new();
    let file_id = files.add(path, source);
    let diagnostic = match e {
        Error::Parse(ParseError::UnrecognizedToken {
            token: (start, tok, end),
            expected,
        }) => Diagnostic::error()
            .with_message(format!("unrecognized token '{}'", tok))
            .with_notes(expected_one_of(expected))
            .with_labels(vec![Label::primary(file_id, start..end)]),
        Error::Parse(ParseError::InvalidToken { location: start }) => Diagnostic::error()
            .with_message("invalid token")
            .with_labels(vec![Label::primary(file_id, start..start + 1)]),
        Error::Parse(_) => Diagnostic::error().with_message(format!("{:?}", e)),
        Error::Runtime(RuntimeError::IdentifierNotFound { name, location }) => Diagnostic::error()
            .with_message(format!("identifier '{}' not found", name))
            .with_labels(vec![Label::primary(file_id, location)]),
        Error::Runtime(RuntimeError::TypeMismatch) => {
            Diagnostic::error().with_message("type mismatch")
        }
        Error::Runtime(RuntimeError::NotCallable { location }) => Diagnostic::error()
            .with_message("not callable")
            .with_labels(vec![Label::primary(file_id, location)]),
        Error::Runtime(RuntimeError::ArityMismatch {
            expected,
            actual,
            location,
        }) => Diagnostic::error()
            .with_message(format!(
                "arity mismatch: expected {} arguments but got {}",
                expected, actual
            ))
            .with_labels(vec![Label::primary(file_id, location)]),
        Error::Assert { location } => Diagnostic::error()
            .with_message("assertion failed")
            .with_labels(vec![Label::primary(file_id, location)]),
    };

    let writer = StandardStream::stderr(ColorChoice::Auto);
    let config = codespan_reporting::term::Config::default();
    let _ = codespan_reporting::term::emit(&mut writer.lock(), &config, &files, &diagnostic);
}

fn expected_one_of(expected: Vec<String>) -> Vec<String> {
    if expected.len() == 1 {
        vec![format!("expected {}", expected[0])]
    } else {
        vec![format!("expected one of {}", expected.join(", "))]
    }
}
