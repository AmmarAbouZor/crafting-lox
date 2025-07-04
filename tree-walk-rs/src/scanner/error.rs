use std::{error::Error, fmt::Display};

use crate::errors::format_err;

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
        format_err(
            f,
            self.line,
            self.pos
                .map(|pos| format!(" {pos}"))
                .unwrap_or_default()
                .as_str(),
            &self.message,
        )
    }
}

impl Error for ScanError {}
