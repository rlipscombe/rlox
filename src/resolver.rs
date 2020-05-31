use crate::ast;
use std::collections::HashMap;

pub fn resolve_locals(statements: &mut Vec<ast::Stmt>) {
    let mut scopes = Vec::new();
    for s in statements {
        resolve_statement(s, &mut scopes);
    }
}

type Scope = HashMap<String, Resolution>;

enum Resolution {
    Declared,
    Defined,
}

fn declare(name: &str, scopes: &mut Vec<Scope>) {
    if !scopes.is_empty() {
        scopes
            .last_mut()
            .unwrap()
            .insert(name.to_string(), Resolution::Declared);
    }
}

fn define(name: &str, scopes: &mut Vec<Scope>) {
    if !scopes.is_empty() {
        scopes
            .last_mut()
            .unwrap()
            .insert(name.to_string(), Resolution::Defined);
    }
}

fn resolve_statement(stmt: &mut ast::Stmt, scopes: &mut Vec<Scope>) {
    use ast::Stmt::*;

    match stmt {
        Block(statements) => {
            scopes.push(Scope::new());
            for s in statements {
                resolve_statement(s, scopes);
            }
            scopes.pop();
        }
        VarDecl { name, init, .. } => {
            declare(name, scopes);
            resolve_expr(init, scopes);
            define(name, scopes);
        }
        FunDecl {
            name, params, body, ..
        } => {
            define(name, scopes);
            scopes.push(Scope::new());
            for p in params {
                define(p, scopes);
            }
            resolve_statement(body, scopes);
            scopes.pop();
        }
        If { cond, then, else_ } => {
            resolve_expr(cond, scopes);
            resolve_statement(then, scopes);
            resolve_statement(else_, scopes);
        }
        While { cond, body, .. } => {
            resolve_expr(cond, scopes);
            resolve_statement(body, scopes);
        }
        Empty => { /* nothing */ }
        Expr(expr) => resolve_expr(expr, scopes),
        Print(expr) => resolve_expr(expr, scopes),
        Assert { expr, .. } => resolve_expr(expr, scopes),
        Return { expr, .. } => resolve_expr(expr, scopes),
    }
}

fn distance_to(name: &str, scopes: &Vec<Scope>) -> Option<usize> {
    for i in (0..scopes.len()).rev() {
        if scopes[i].contains_key(name) {
            return Some(i);
        }
    }

    None
}

fn resolve_expr(expr: &mut ast::Expr, scopes: &mut Vec<Scope>) {
    use ast::Expr::*;

    match expr {
        Var {
            name,
            ref mut distance,
            ..
        } => match scopes.last() {
            Some(top) => match top.get(name) {
                Some(Resolution::Declared) => {
                    panic!("reading local variable in its own initializer")
                }
                _ => {
                    *distance = distance_to(name, scopes);
                }
            },
            None => { /* global scope; nothing to do */ }
        },
        Assignment {
            name,
            rhs,
            ref mut distance,
            ..
        } => {
            resolve_expr(rhs, scopes);
            *distance = distance_to(name, scopes);
        }
        Binary { left, right, .. } => {
            resolve_expr(left, scopes);
            resolve_expr(right, scopes);
        }
        Call { callee, args, .. } => {
            resolve_expr(callee, scopes);
            for a in args {
                resolve_expr(a, scopes);
            }
        }
        Fun { params, body, .. } => {
            scopes.push(Scope::new());
            for p in params {
                define(p, scopes);
            }
            resolve_statement(body, scopes);
            scopes.pop();
        }
        Unary { right, .. } => resolve_expr(right, scopes),
        Nil { .. } => {}
        Number { .. } => {}
        Boolean { .. } => {}
        String { .. } => {}
    }
}
