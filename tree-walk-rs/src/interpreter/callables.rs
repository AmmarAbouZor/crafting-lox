use std::{fmt::Display, time::SystemTime};

use crate::{RuntimeError, Token, TokenType};

use super::{Interpreter, LoxValue};

type Result<T> = std::result::Result<T, RuntimeError>;

pub const CLOCK_NAME: &str = "clock";

// TODO: Implementing all the traits isn't necessary if we want to move
// out from LoxValue
#[derive(Debug, Clone, PartialEq)]
pub enum LoxCallable {
    Clock,
}

impl LoxCallable {
    pub fn call(
        &self,
        _interprerter: &mut Interpreter,
        _arguments: &[LoxValue],
    ) -> Result<LoxValue> {
        match self {
            LoxCallable::Clock => Self::clock(),
        }
    }

    pub fn arity(&self) -> usize {
        match self {
            LoxCallable::Clock => 0,
        }
    }

    fn clock() -> Result<LoxValue> {
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|t| t.as_secs())
            .map(|t| LoxValue::Number(t as f64))
            .map_err(|err| {
                RuntimeError::new(
                    Token::new(TokenType::Fun, CLOCK_NAME, 0),
                    format!("Error while calling system time: {err}"),
                )
            })
    }
}

impl Display for LoxCallable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoxCallable::Clock => f.write_str("<native fn>"),
        }
    }
}
