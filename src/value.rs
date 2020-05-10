use crate::ast;
use crate::environment::Environment;

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
