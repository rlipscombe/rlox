#[derive(Debug, PartialEq)]
pub enum Expr {
    Nil,
    Number(f64),
    Boolean(bool),
    String(String),
    Unary(UnaryOp, Box<Expr>),
    Binary(Box<Expr>, BinaryOp, Box<Expr>),
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
}
