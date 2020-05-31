#[macro_use]
extern crate lalrpop_util;
lalrpop_mod!(pub lox);

mod ast;
mod environment;
mod resolver;
mod interpreter;
mod bindings;
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
    bindings::register_globals(&mut environment);
    match interpret_source(&source, &mut environment) {
        Ok(_) => {}
        Err(e) => error::report_error(&path, &source, e),
    };
}
