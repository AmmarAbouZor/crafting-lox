use thiserror::Error;

#[derive(Error, Debug)]
/// General error for rlox interpreter.
pub enum RunError {
    #[error("Unrecoverable error in rlox: {0}")]
    Unrecoverable(#[from] anyhow::Error),
    #[error("Scanning failed with {0} errors")]
    Scann(usize),
}
