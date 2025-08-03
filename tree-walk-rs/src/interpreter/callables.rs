use std::{cell::RefCell, fmt::Display, rc::Rc, time::SystemTime};

use crate::{RuntimeError, Token, TokenType};

use super::{Interpreter, LoxValue, class::LoxClass, function::LoxFunction};

type Result<T> = std::result::Result<T, RuntimeError>;

pub type LoxClassRef = Rc<RefCell<LoxClass>>;

pub const CLOCK_NAME: &str = "clock";

// TODO: Implementing all the traits isn't necessary if we want to move
// out from LoxValue
#[derive(Debug, Clone, PartialEq)]
pub enum LoxCallable {
    Clock,
    LoxFunction(LoxFunction),
    Class(LoxClassRef),
}

impl LoxCallable {
    pub fn call(&self, interprerter: &mut Interpreter, arguments: &[LoxValue]) -> Result<LoxValue> {
        match self {
            LoxCallable::Clock => Self::clock(),
            LoxCallable::LoxFunction(func) => func.call(interprerter, arguments),
            LoxCallable::Class(lox_class) => lox_class.borrow().call(interprerter, arguments),
        }
    }

    pub fn arity(&self) -> usize {
        match self {
            LoxCallable::Clock => 0,
            LoxCallable::LoxFunction(func) => func.arity(),
            LoxCallable::Class(lox_class) => lox_class.borrow().arity(),
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
            LoxCallable::LoxFunction(func) => {
                write!(f, "{func}")
            }
            LoxCallable::Class(lox_class) => {
                write!(f, "{}", lox_class.borrow())
            }
        }
    }
}
