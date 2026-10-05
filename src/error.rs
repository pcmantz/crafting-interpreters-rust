/* error.rs
 *
 */

use crate::prelude::*;

use crate::token::*;

#[derive(Debug, Clone)]
pub struct Error {
    kind: ErrorKind,
    span: Span,
}

#[derive(Debug, Clone)]
pub enum ErrorKind {
    ScannerError {
        message: String,
    },
    WrongToken {
        expected: TokenType,
        found: TokenType,
    },
    MissingExpression {
        message: String,
    },
    Runtime {
        message: String,
    },
    InvalidAssignment {},
    TooManyArguments {},
    ValueNotCallable {
        // called: Token,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[offset: {}] Error: ", self.span.start.offset)?;

        match &self.kind {
            ErrorKind::WrongToken { expected, found } => {
                write!(f, "Wrong token. expected: {}, found: {}", expected, found)
            }
            ErrorKind::ScannerError { message } | ErrorKind::MissingExpression { message } => {
                write!(f, "{}", message)
            }
            ErrorKind::Runtime { message, .. } => write!(f, "{}", message),
            ErrorKind::InvalidAssignment { .. } => write!(f, "Invalid assignment."),
            ErrorKind::TooManyArguments { .. } => write!(f, "Too many arguments to function."),
            ErrorKind::ValueNotCallable { .. } => write!(f, "Value not callable."),
        }
    }
}

impl std::error::Error for Error {}

impl Error {
    pub fn scanner(message: impl Into<String>, span: Span) -> Error {
        Error {
            kind: ErrorKind::ScannerError {
                message: message.into(),
            },
            span,
        }
    }

    pub fn wrong_token(expected: TokenType, found: &Token) -> Error {
        Error {
            kind: ErrorKind::WrongToken {
                expected,
                found: found.ty.clone(),
            },
            span: found.span,
        }
    }

    pub fn missing_expression(token: &Token, message: impl Into<String>) -> Error {
        Error {
            kind: ErrorKind::MissingExpression {
                message: message.into(),
            },
            span: token.span,
        }
    }

    pub fn runtime(token: &Token, message: impl Into<String>) -> Error {
        Error {
            kind: ErrorKind::Runtime {
                message: message.into(),
            },
            span: token.span,
        }
    }

    pub fn invalid_assignment(span: Span) -> Error {
        Error {
            kind: ErrorKind::InvalidAssignment {},
            span,
        }
    }

    pub fn too_many_arguments(span: Span) -> Error {
        Error {
            kind: ErrorKind::TooManyArguments {},
            span,
        }
    }

    pub fn value_not_callable(span: Span) -> Error {
        Error {
            kind: ErrorKind::ValueNotCallable {
                // called // This should be the start of an expr
            },
            span,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ParseErrors(pub Vec<Error>);

impl fmt::Display for ParseErrors {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0.iter().join("\n"))
    }
}

impl std::error::Error for ParseErrors {}
