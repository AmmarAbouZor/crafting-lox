use std::{error::Error, fmt::Display};

/// Error while scanning code.
#[derive(Debug)]
pub struct ScannError {
    line: usize,
    message: String,
    pos: Option<usize>,
}

impl ScannError {
    pub fn new(line: usize, message: String, pos: Option<usize>) -> Self {
        Self { line, message, pos }
    }
}

impl Display for ScannError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[line {line}] Error{pos}: {message}",
            line = self.line,
            message = self.message,
            pos = self.pos.map(|pos| format!(" {pos}")).unwrap_or_default()
        )
    }
}

impl Error for ScannError {}
