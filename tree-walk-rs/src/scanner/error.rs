use std::{error::Error, fmt::Display};

/// Error while scanning code.
#[derive(Debug)]
pub struct ScanError {
    line: usize,
    message: String,
    pos: Option<usize>,
}

impl ScanError {
    pub fn new(line: usize, message: impl Into<String>, pos: Option<usize>) -> Self {
        Self {
            line,
            message: message.into(),
            pos,
        }
    }
}

impl Display for ScanError {
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

impl Error for ScanError {}
