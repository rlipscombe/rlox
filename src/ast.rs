#[derive(Debug)]
pub enum Expr {
    Nil,
    Number(f64),
    Boolean(bool),
    Unary(UnaryOp, Box<Expr>),
    Binary(Box<Expr>, BinaryOp, Box<Expr>),
}

#[derive(Debug)]
pub enum UnaryOp {
    Invert,
    Negate,
}

#[derive(Debug)]
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
