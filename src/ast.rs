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
    Nil {
        location: Location,
    },
    Number {
        value: f64,
        location: Location,
    },
    Boolean {
        value: bool,
        location: Location,
    },
    String {
        value: String,
        location: Location,
    },
    Unary {
        op: UnaryOp,
        right: Box<Expr>,
        location: Location,
    },
    Binary {
        left: Box<Expr>,
        op: BinaryOp,
        right: Box<Expr>,
        location: Location,
    },
    Var {
        name: String,
        location: Location,
    },
    Assignment {
        name: String,
        rhs: Box<Expr>,
        location: Location,
    },
    Call {
        callee: Box<Expr>,
        location: Location,
    },
}

pub trait Locatable {
    fn location(&self) -> Location;
}

impl Locatable for Expr {
    fn location(&self) -> Location {
        match *self {
            Expr::Nil { location, ..} => location,
            Expr::Number { location, ..} => location,
            Expr::Boolean { location, ..} => location,
            Expr::String { location, ..} => location,
            Expr::Unary { location, ..} => location,
            Expr::Binary { location, ..} => location,
            Expr::Var { location, ..} => location,
            Expr::Assignment { location, ..} => location,
            Expr::Call { location, .. } => location,
        }
    }
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

pub fn desugar_for(
    location: Location,
    init: Option<Stmt>,
    cond: Option<Expr>,
    incr: Option<Expr>,
    body: Stmt,
) -> Stmt {
    desugar_for_(
        init.unwrap_or(Stmt::Empty),
        cond.unwrap_or(Expr::Boolean {
            value: true,
            location: location,
        }),
        incr.unwrap_or(Expr::Nil { location: location }),
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
