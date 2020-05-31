use codespan_reporting::diagnostic::{Diagnostic, Label};
use codespan_reporting::files::SimpleFiles;
use codespan_reporting::term::termcolor::{ColorChoice, StandardStream};

use crate::ast;
use crate::lox;
use crate::Value;

type ParseError<'s> = lalrpop_util::ParseError<usize, lox::Token<'s>, &'s str>;

#[derive(Debug, PartialEq)]
pub enum Error<'s> {
    Parse(ParseError<'s>),
    Runtime(RuntimeError),
    Internal { file: &'static str, line: u32, location: ast::Location },
    Assert { location: ast::Location },
    Return(Value),
}

#[derive(Debug, PartialEq)]
pub enum RuntimeError {
    TypeMismatch,
    IdentifierNotFound {
        name: String,
        location: ast::Location,
    },
    NotCallable {
        location: ast::Location,
    },
    ArityMismatch {
        expected: usize,
        actual: usize,
        location: ast::Location,
    },
}

pub fn report_error(path: &str, source: &str, e: Error) {
    let mut files = SimpleFiles::new();
    let file_id = files.add(path, source);
    let diagnostic = match e {
        Error::Parse(ParseError::UnrecognizedToken {
            token: (start, tok, end),
            expected,
        }) => Diagnostic::error()
            .with_message(format!("unrecognized token '{}'", tok))
            .with_notes(expected_one_of(expected))
            .with_labels(vec![Label::primary(file_id, start..end)]),
        Error::Parse(ParseError::InvalidToken { location: start }) => Diagnostic::error()
            .with_message("invalid token")
            .with_labels(vec![Label::primary(file_id, start..start + 1)]),
        Error::Parse(_) => Diagnostic::error().with_message(format!("{:?}", e)),
        Error::Runtime(RuntimeError::IdentifierNotFound { name, location }) => Diagnostic::error()
            .with_message(format!("identifier '{}' not found", name))
            .with_labels(vec![Label::primary(file_id, location)]),
        Error::Runtime(RuntimeError::TypeMismatch) => {
            Diagnostic::error().with_message("type mismatch")
        }
        Error::Runtime(RuntimeError::NotCallable { location }) => Diagnostic::error()
            .with_message("not callable")
            .with_labels(vec![Label::primary(file_id, location)]),
        Error::Runtime(RuntimeError::ArityMismatch {
            expected,
            actual,
            location,
        }) => Diagnostic::error()
            .with_message(format!(
                "arity mismatch: expected {} arguments but got {}",
                expected, actual
            ))
            .with_labels(vec![Label::primary(file_id, location)]),
        Error::Internal { file, line, location } => Diagnostic::error()
            .with_message(format!("INTERNAL ERROR {}:{}", file, line))
            .with_labels(vec![Label::primary(file_id, location)]),
        Error::Assert { location } => Diagnostic::error()
            .with_message("assertion failed")
            .with_labels(vec![Label::primary(file_id, location)]),
        Error::Return(_) => panic!("using error for return values was a bad idea?"),
    };

    let writer = StandardStream::stderr(ColorChoice::Auto);
    let config = codespan_reporting::term::Config::default();
    let _ = codespan_reporting::term::emit(&mut writer.lock(), &config, &files, &diagnostic);
}

fn expected_one_of(expected: Vec<String>) -> Vec<String> {
    if expected.len() == 1 {
        vec![format!("expected {}", expected[0])]
    } else {
        vec![format!("expected one of {}", expected.join(", "))]
    }
}
