#[derive(Debug)]
pub enum Expr {
    Number(f64),
    Binary(Box<Expr>, BinaryOp, Box<Expr>),
}

#[derive(Debug)]
pub enum BinaryOp {
    Mul,
    Div,
    Add,
    Sub,
}
