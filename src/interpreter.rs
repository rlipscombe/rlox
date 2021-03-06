use crate::lox;
use crate::environment::Environment;
use crate::error::*;
use crate::value::Value;
use crate::ast;
use crate::ast::Locatable;

pub fn interpret_source<'s>(source: &'s str, environment: &mut Environment) -> Result<(), Error<'s>> {
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
        VarDecl { name, init, .. } => {
            let value = evaluate(&init, environment)?;
            environment.define(&name, value);
            Ok(())
        }
        FunDecl {
            name, params, body, ..
        } => {
            // BUG: Because 'clone' merely clones the Rc, we end up with
            // another reference to the same environment. Which means that
            // when we declare a new variable in it...
            // If we'd nested it slightly more, we'd have an enclosed environment
            // and that would have done the right thing. I think.
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
            // This should create a new environment, so that 'var' is scoped correctly.
            // But: does that break anything because we use Block for a few other things?
            let mut environment = Environment::with_enclosing(environment);
            let result = interpret_statements(statements, &mut environment);
            result
        }
        Return { expr, .. } => {
            // TODO: I'm uneasy about using Result for this; it feels like we need
            // something specific to this use.
            let value = evaluate(expr, environment)?;
            Err(Error::Return(value))
        }
        If { cond, then, else_ } => match evaluate(&cond, environment) {
            Ok(Value::Boolean(true)) => interpret_statement(then, environment),
            Ok(Value::Boolean(false)) => interpret_statement(else_, environment),
            Ok(_) => Err(Error::Runtime(RuntimeError::TypeMismatch { location: cond.location() })),
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
                    return Err(Error::Runtime(RuntimeError::TypeMismatch { location: cond.location() }));
                }
                Err(e) => return Err(e),
            }
        },
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
                _ => Err(Error::Runtime(RuntimeError::TypeMismatch { location: right.location() })),
            },
            ast::UnaryOp::Negate => match evaluate(&right, environment)? {
                Value::Number(n) => Ok(Value::Number(-n)),
                _ => Err(Error::Runtime(RuntimeError::TypeMismatch { location: right.location() })),
            },
        },
        ast::Expr::Binary {
            left, op, right, ..
        } => match op {
            ast::BinaryOp::Add => do_add(&left, &right, environment),
            ast::BinaryOp::Sub => do_sub(&left, &right, environment),
            ast::BinaryOp::Mul => do_mul(&left, &right, environment),
            ast::BinaryOp::Div => do_div(&left, &right, environment),
            ast::BinaryOp::Mod => do_mod(&left, &right, environment),
            ast::BinaryOp::Eq => do_eq(&left, &right, environment),
            ast::BinaryOp::Ne => do_ne(&left, &right, environment),
            ast::BinaryOp::Lt => do_lt(&left, &right, environment),
            ast::BinaryOp::Le => do_le(&left, &right, environment),
            ast::BinaryOp::Gt => do_gt(&left, &right, environment),
            ast::BinaryOp::Ge => do_ge(&left, &right, environment),
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
        ast::Expr::Fun {
            params,
            body,
            location,
            ..
        } => {
            let closure = environment.clone();
            Ok(Value::LoxFunction {
                name: format!("<anon@{}>", location.start),
                closure: closure,
                params: params.to_vec(),
                body: body.clone(),
            })
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
        NativeFunction { name, .. } => println!("<fun {} (native)>", name),
        LoxFunction { name, .. } => println!("<fun {} (lox)>", name),
    }
}

fn do_add<'s>(lhs: &ast::Expr, rhs: &ast::Expr, environment: &mut Environment) -> Result<Value, Error<'s>> {
    let lv = evaluate(lhs, environment)?;
    let rv = evaluate(rhs, environment)?;
    match (lv, rv) {
        (Value::Number(l), Value::Number(r)) => Ok(Value::Number(l + r)),
        (Value::String(l), Value::String(r)) => Ok(Value::String(l + &r)),
        _ => Err(Error::Runtime(RuntimeError::TypeMismatch { location: rhs.location() })),
    }
}

fn do_sub<'s>(lhs: &ast::Expr, rhs: &ast::Expr, environment: &mut Environment) -> Result<Value, Error<'s>> {
    let lv = evaluate(lhs, environment)?;
    let rv = evaluate(rhs, environment)?;
    match (lv, rv) {
        (Value::Number(l), Value::Number(r)) => Ok(Value::Number(l - r)),
        _ => Err(Error::Runtime(RuntimeError::TypeMismatch { location: rhs.location() })),
    }
}

fn do_mul<'s>(lhs: &ast::Expr, rhs: &ast::Expr, environment: &mut Environment) -> Result<Value, Error<'s>> {
    let lv = evaluate(lhs, environment)?;
    let rv = evaluate(rhs, environment)?;
    match (lv, rv) {
        (Value::Number(l), Value::Number(r)) => Ok(Value::Number(l * r)),
        _ => Err(Error::Runtime(RuntimeError::TypeMismatch { location: rhs.location() })),
    }
}

fn do_div<'s>(lhs: &ast::Expr, rhs: &ast::Expr, environment: &mut Environment) -> Result<Value, Error<'s>> {
    let lv = evaluate(lhs, environment)?;
    let rv = evaluate(rhs, environment)?;
    match (lv, rv) {
        (Value::Number(l), Value::Number(r)) => Ok(Value::Number(l / r)),
        _ => Err(Error::Runtime(RuntimeError::TypeMismatch { location: rhs.location() })),
    }
}

fn do_mod<'s>(lhs: &ast::Expr, rhs: &ast::Expr, environment: &mut Environment) -> Result<Value, Error<'s>> {
    let lv = evaluate(lhs, environment)?;
    let rv = evaluate(rhs, environment)?;
    match (lv, rv) {
        (Value::Number(l), Value::Number(r)) => Ok(Value::Number(l % r)),
        _ => Err(Error::Runtime(RuntimeError::TypeMismatch { location: rhs.location() })),
    }
}

fn do_eq<'s>(lhs: &ast::Expr, rhs: &ast::Expr, environment: &mut Environment) -> Result<Value, Error<'s>> {
    let lv = evaluate(lhs, environment)?;
    let rv = evaluate(rhs, environment)?;
    match (lv, rv) {
        (Value::Number(l), Value::Number(r)) => Ok(Value::Boolean(l == r)),
        (Value::String(l), Value::String(r)) => Ok(Value::Boolean(l == r)),
        (Value::Boolean(l), Value::Boolean(r)) => Ok(Value::Boolean(l == r)),
        (Value::Nil, Value::Nil) => Ok(Value::Boolean(true)),
        // Types don't match => false
        _ => Ok(Value::Boolean(false)),
    }
}

fn do_ne<'s>(lhs: &ast::Expr, rhs: &ast::Expr, environment: &mut Environment) -> Result<Value, Error<'s>> {
    let lv = evaluate(lhs, environment)?;
    let rv = evaluate(rhs, environment)?;
    match (lv, rv) {
        (Value::Number(l), Value::Number(r)) => Ok(Value::Boolean(l != r)),
        (Value::String(l), Value::String(r)) => Ok(Value::Boolean(l != r)),
        (Value::Boolean(l), Value::Boolean(r)) => Ok(Value::Boolean(l != r)),
        (Value::Nil, Value::Nil) => Ok(Value::Boolean(false)),
        // Types don't match => true
        _ => Ok(Value::Boolean(true)),
    }
}

fn do_lt<'s>(lhs: &ast::Expr, rhs: &ast::Expr, environment: &mut Environment) -> Result<Value, Error<'s>> {
    let lv = evaluate(lhs, environment)?;
    let rv = evaluate(rhs, environment)?;
    match (lv, rv) {
        (Value::Number(l), Value::Number(r)) => Ok(Value::Boolean(l < r)),
        _ => Err(Error::Runtime(RuntimeError::TypeMismatch { location: rhs.location() })),
    }
}

fn do_le<'s>(lhs: &ast::Expr, rhs: &ast::Expr, environment: &mut Environment) -> Result<Value, Error<'s>> {
    let lv = evaluate(lhs, environment)?;
    let rv = evaluate(rhs, environment)?;
    match (lv, rv) {
        (Value::Number(l), Value::Number(r)) => Ok(Value::Boolean(l <= r)),
        _ => Err(Error::Runtime(RuntimeError::TypeMismatch { location: rhs.location() })),
    }
}

fn do_gt<'s>(lhs: &ast::Expr, rhs: &ast::Expr, environment: &mut Environment) -> Result<Value, Error<'s>> {
    let lv = evaluate(lhs, environment)?;
    let rv = evaluate(rhs, environment)?;
    match (lv, rv) {
        (Value::Number(l), Value::Number(r)) => Ok(Value::Boolean(l > r)),
        _ => Err(Error::Runtime(RuntimeError::TypeMismatch { location: rhs.location() })),
    }
}

fn do_ge<'s>(lhs: &ast::Expr, rhs: &ast::Expr, environment: &mut Environment) -> Result<Value, Error<'s>> {
    let lv = evaluate(lhs, environment)?;
    let rv = evaluate(rhs, environment)?;
    match (lv, rv) {
        (Value::Number(l), Value::Number(r)) => Ok(Value::Boolean(l >= r)),
        _ => Err(Error::Runtime(RuntimeError::TypeMismatch { location: rhs.location() })),
    }
}

fn do_call<'s>(
    callee: &ast::Expr,
    args: &Vec<ast::Expr>,
    environment: &mut Environment,
) -> Result<Value, Error<'s>> {
    // The Java reference implementation of Lox evaluates the callee,
    // then the arguments, and _then_ checks that the callee is actually
    // callable. This could be important, since evaluating the callee and the
    // arguments might have side-effects. As it turns out, I don't think it
    // _is_ important, since calling a non-callable would be a runtime error
    // and that actually aborts the program, and we don't have any externally-visible
    // side-effects (other than print, so /shrug)
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
            match interpret_statement(&body, &mut environment) {
                Ok(()) => Ok(Value::Nil),
                Err(Error::Return(value)) => Ok(value),
                Err(e) => Err(e),
            }
        }
        _ => {
            let location = callee.location();
            Err(Error::Runtime(RuntimeError::NotCallable {
                location: location,
            }))
        }
    }
}
