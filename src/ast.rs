#[derive(Debug)]
pub enum Expr {
    Nil,
    Number(f64),
    Boolean(bool),
    Binary(Box<Expr>, BinaryOp, Box<Expr>),
}

#[derive(Debug)]
pub enum BinaryOp {
    Mul,
    Div,
    Add,
    Sub,

    Lt,
    Le,
    Gt,
    Ge,
}
