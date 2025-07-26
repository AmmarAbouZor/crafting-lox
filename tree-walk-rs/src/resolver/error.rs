use std::{borrow::Cow, error::Error, fmt::Display};

use crate::{Token, TokenType, errors::format_err};

// Copied from ParseError
// TODO: Unify the duplicated errors if they stay duplicated.
#[derive(Debug)]
pub struct ResolveError {
    token: Token,
    msg: String,
}

impl ResolveError {
    pub fn new(token: Token, msg: impl Into<String>) -> Self {
        Self {
            token,
            msg: msg.into(),
        }
    }
}

impl Display for ResolveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let pos = if self.token.typ == TokenType::Eof {
            Cow::Borrowed(" at end")
        } else {
            Cow::Owned(format!(" at '{}'", self.token.lexeme))
        };

        format_err(f, self.token.line, &pos, &self.msg)
    }
}

impl Error for ResolveError {}
