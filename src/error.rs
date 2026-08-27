/* error.rs
 *
 */

use crate::prelude::*;

use crate::token::*;

#[derive(Debug, Clone)]
pub struct Error {
    kind: ErrorKind,
    line: usize,
    col: i64,
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
        token: Token,
        message: String,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[line {}:{}] Error: ", self.line, self.col)?;

        match &self.kind {
            ErrorKind::WrongToken { expected, found } => {
                write!(f, "Wrong token. expected: {}, found: {}", expected, found)
            }
            ErrorKind::ScannerError { message } | ErrorKind::MissingExpression { message } => {
                write!(f, "{}", message)
            }
            ErrorKind::Runtime { message, .. } => write!(f, "{}", message),
        }
    }
}

impl std::error::Error for Error {}

impl Error {
    pub fn scanner(message: impl Into<String>, line: usize, col: i64) -> Error {
        Error {
            kind: ErrorKind::ScannerError {
                message: message.into(),
            },
            line,
            col,
        }
    }

    pub fn wrong_token(expected: TokenType, found: &Token) -> Error {
        Error {
            kind: ErrorKind::WrongToken {
                expected,
                found: found.ty.clone(),
            },
            line: found.line,
            col: found.col,
        }
    }

    pub fn missing_expression(token: &Token, message: impl Into<String>) -> Error {
        Error {
            kind: ErrorKind::MissingExpression {
                message: message.into(),
            },
            line: token.line,
            col: token.col,
        }
    }

    pub fn runtime(token: &Token, message: impl Into<String>) -> Error {
        Error {
            kind: ErrorKind::Runtime {
                token: token.clone(),
                message: message.into(),
            },
            line: token.line,
            col: token.col,
        }
    }
}
