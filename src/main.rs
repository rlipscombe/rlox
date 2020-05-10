#[macro_use]
extern crate lalrpop_util;
lalrpop_mod!(pub lox);

use std::time::{SystemTime, UNIX_EPOCH};

mod ast;
mod environment;
mod interpreter;
mod error;
mod value;

mod test;

use environment::Environment;
use interpreter::interpret_source;
use value::Value;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() <= 1 {
        eprintln!("{} file.lox", args[0]);
        return;
    }

    let path = &args[1];
    let source = std::fs::read_to_string(path).expect("read file");

    let mut environment = Environment::new();
    let clock = Value::NativeFunction {
        name: "clock".to_string(),
        arity: 0,
        fun: |_argv| {
            let now = SystemTime::now();
            Value::Number(now.duration_since(UNIX_EPOCH).unwrap().as_millis() as f64)
        },
    };
    environment.define("clock", clock);
    match interpret_source(&source, &mut environment) {
        Ok(_) => {}
        Err(e) => error::report_error(&path, &source, e),
    };
}
