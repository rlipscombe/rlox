# rlox

Rust implementation of Lox from ["Crafting Interpreters"](http://craftinginterpreters.com).

## Progress

It's complete (as far as I can tell) up to the end of Chapter 10.

**Next:** Chapter 11: ["Resolving and Binding"](http://craftinginterpreters.com/resolving-and-binding.html).

## Building it

Build it with `cargo build`. Run it with (e.g.) `cargo run -- examples/fibonacci-recursive.lox`. Run the tests with
`cargo test`.

## Differences from the reference implementation

* Added a modulo (%) operator.
* Anonymous functions.

![Rust](https://github.com/rlipscombe/rlox/workflows/Rust/badge.svg)
