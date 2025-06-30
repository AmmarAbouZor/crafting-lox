use thiserror::Error;

use crate::scanner::ScannError;

#[derive(Error, Debug)]
/// General error for rlox interpreter.
pub enum RunError {
    #[error("Unrecoverable error in rlox: {0}")]
    Unrecoverable(#[from] anyhow::Error),
    #[error("{0}")]
    ScannError(#[from] ScannError),
}
