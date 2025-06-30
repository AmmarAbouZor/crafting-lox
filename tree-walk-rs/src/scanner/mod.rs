use std::fmt::Display;

mod error;

pub use error::ScannError;

pub struct Scanner {
    text: String,
}

impl Scanner {
    pub fn new(text: String) -> Self {
        Self { text }
    }

    pub fn scan_tokens(&mut self) -> Result<Vec<Token>, ScannError> {
        Ok(vec![])
    }
}

#[derive(Debug, Clone)]
pub enum Token {}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "We don't have tokens yet")
    }
}
