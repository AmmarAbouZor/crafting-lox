use std::{error::Error, fmt::Display};

use crate::Token;

use super::LoxValue;

#[derive(Debug)]
pub enum RuntimeError {
    Error { token: Token, message: String },
    // TODO: I think Error is misused here for return statements.
    // For now I'll keep it like this to continue with the book but
    // I should look into other solutions once the first part is done.
    Return { value: LoxValue },
}

impl RuntimeError {
    pub fn new(token: Token, message: impl Into<String>) -> Self {
        Self::Error {
            token,
            message: message.into(),
        }
    }
}

impl Display for RuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RuntimeError::Error { token, message } => {
                writeln!(f, "{message}")?;
                write!(f, "[line {}]", token.line)
            }
            RuntimeError::Return { value } => {
                write!(f, "Return value: {value}")
            }
        }
    }
}

impl Error for RuntimeError {}
