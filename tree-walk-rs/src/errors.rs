use thiserror::Error;

#[derive(Error, Debug)]
/// General error for rlox interpreter.
pub enum RunError {
    #[error("Unrecoverable error in rlox: {0}")]
    Unrecoverable(#[from] anyhow::Error),
    #[error("Scanning failed with {0} errors")]
    Scann(usize),
}

use std::fmt::{Formatter, Result};

pub fn format_err(f: &mut Formatter<'_>, line: usize, position: &str, message: &str) -> Result {
    write!(f, "[line {line}] Error{position}: {message}",)
}
