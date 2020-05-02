#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Location {
    pub start: usize,
    pub end: usize,
}

impl Into<std::ops::Range<usize>> for Location {
    fn into(self) -> std::ops::Range<usize> {
        self.start..self.end
    }
}

#[derive(Debug, PartialEq)]
pub enum Expr {
    Nil,
    Number(f64),
    Boolean(bool),
    String(String),
    Unary(UnaryOp, Box<Expr>),
    Binary(Box<Expr>, BinaryOp, Box<Expr>),
    Var {
        name: String,
        location: Location,
    },
    Assignment {
        name: String,
        rhs: Box<Expr>,
        location: Location,
    },
    Call(Box<Expr>)
}

#[derive(Debug, PartialEq)]
pub enum UnaryOp {
    Invert,
    Negate,
}

#[derive(Debug, PartialEq)]
pub enum BinaryOp {
    Mul,
    Div,
    Mod,

    Add,
    Sub,

    Eq,
    Ne,

    Lt,
    Le,
    Gt,
    Ge,
}

#[derive(Debug, PartialEq)]
pub enum Stmt {
    Empty,
    Expr(Expr),
    Print(Expr),
    Assert {
        expr: Expr,
        location: Location,
    },
    VarDecl(String, Expr),
    Block(Vec<Stmt>),
    If {
        cond: Expr,
        then: Box<Stmt>,
        else_: Box<Stmt>,
    },
    While {
        cond: Expr,
        body: Box<Stmt>,
    },
}

pub fn location(s: usize, e: usize) -> Location {
    Location { start: s, end: e }
}

pub fn desugar_for(init: Option<Stmt>, cond: Option<Expr>, incr: Option<Expr>, body: Stmt) -> Stmt {
    desugar_for_(
        init.unwrap_or(Stmt::Empty),
        cond.unwrap_or(Expr::Boolean(true)),
        incr.unwrap_or(Expr::Nil),
        body,
    )
}

pub fn desugar_for_(init: Stmt, cond: Expr, incr: Expr, body: Stmt) -> Stmt {
    Stmt::Block(vec![
        init,
        Stmt::While {
            cond: cond,
            body: Box::new(Stmt::Block(vec![body, Stmt::Expr(incr)])),
        },
    ])
}
