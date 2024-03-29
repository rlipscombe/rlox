#[macro_use]
extern crate lalrpop_util;
lalrpop_mod!(pub lox);

mod ast;
mod bindings;
mod environment;
mod error;
mod interpreter;
mod value;

mod test;

use environment::Environment;
use interpreter::interpret_source;
use value::Value;

use clap::Parser;

#[derive(Parser)]
struct Opts {
    #[clap(value_parser)]
    input: String,

    #[clap(long, action)]
    simple_errors: bool,
}

fn main() {
    let opts = Opts::parse();

    let source = std::fs::read_to_string(&opts.input).expect("read file");

    let mut environment = Environment::new();
    bindings::register_globals(&mut environment);
    match interpret_source(&source, &mut environment) {
        Ok(_) => {}
        Err(e) => error::report_error(&opts.input, &source, opts.simple_errors, e),
    };
}
