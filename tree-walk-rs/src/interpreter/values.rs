use std::fmt::Display;

use crate::ast::LiteralValue;

use super::{callables::LoxCallable, instance::LoxInstance};

#[derive(Debug, Clone, PartialEq)]
pub enum LoxValue {
    Nil,
    Boolean(bool),
    Number(f64),
    String(String),
    Callable(LoxCallable),
    Instance(LoxInstance),
}

impl From<&LiteralValue> for LoxValue {
    fn from(value: &LiteralValue) -> Self {
        match value {
            LiteralValue::Nil => LoxValue::Nil,
            LiteralValue::Boolean(val) => LoxValue::Boolean(*val),
            LiteralValue::Text(val) => LoxValue::String(val.into()),
            LiteralValue::Number(val) => LoxValue::Number(*val),
        }
    }
}

impl Display for LoxValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoxValue::Nil => f.write_str("Nil"),
            LoxValue::Boolean(val) => write!(f, "{val}"),
            LoxValue::Number(val) => write!(f, "{val}"),
            LoxValue::String(val) => write!(f, "{val}"),
            LoxValue::Callable(lox_callable) => write!(f, "{lox_callable}"),
            LoxValue::Instance(instance) => write!(f, "{instance}"),
        }
    }
}

impl LoxValue {
    pub fn is_truthy(&self) -> bool {
        // We follow Ruby approach in Lox
        match self {
            LoxValue::Nil => false,
            LoxValue::Boolean(val) => *val,
            LoxValue::Number(..)
            | LoxValue::String(..)
            | LoxValue::Callable(..)
            | LoxValue::Instance(..) => true,
        }
    }
}
