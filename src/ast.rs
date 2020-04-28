#[derive(Debug, PartialEq)]
pub struct Location {
    pub start: usize,
    pub end: usize,
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
    Expr(Box<Expr>),
    Print(Box<Expr>),
    VarDecl(String, Box<Expr>),
    Block(Vec<Stmt>),
}

pub fn location(s: usize, e: usize) -> Location {
    Location { start: s, end: e }
}
