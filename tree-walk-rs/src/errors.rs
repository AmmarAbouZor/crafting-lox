use thiserror::Error;

use crate::{LoxValue, Token};

pub type LoxResult<T> = std::result::Result<T, LoxError>;

#[derive(Error, Debug)]
/// General error for rlox interpreter.
pub enum RunError {
    #[error("Unrecoverable error in rlox: {0}")]
    Unrecoverable(#[from] anyhow::Error),
    #[error("Scanning failed with {0} errors")]
    Scan(usize),
    #[error("{0}")]
    LoxError(#[from] LoxError),
}

use std::fmt::{Display, Formatter, Result};
pub fn format_err(f: &mut Formatter<'_>, line: usize, position: &str, message: &str) -> Result {
    write!(f, "[line {line}] Error{position}: {message}",)
}

#[derive(Debug)]
pub enum LoxError {
    Error { token: Token, message: String },
    // TODO: I think Error is misused here for return statements.
    // For now I'll keep it like this to continue with the book but
    // I should look into other solutions once the first part is done.
    Return { value: Box<LoxValue> },
}

impl LoxError {
    pub fn new(token: Token, message: impl Into<String>) -> Self {
        Self::Error {
            token,
            message: message.into(),
        }
    }
}

impl Display for LoxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoxError::Error { token, message } => {
                writeln!(f, "{message}")?;
                write!(f, "[line {}]", token.line)
            }
            LoxError::Return { value } => {
                write!(f, "Return value: {value}")
            }
        }
    }
}

impl std::error::Error for LoxError {}
