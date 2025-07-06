mod expression;
mod statement;

use std::fmt::Display;

pub use expression::Expr;
pub use statement::Stmt;

#[derive(Debug, Clone)]
pub enum LiteralValue {
    Nil,
    Boolean(bool),
    Text(String),
    Number(f64),
}

impl Display for LiteralValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LiteralValue::Nil => f.write_str("Nil"),
            LiteralValue::Boolean(bool) => write!(f, "{bool}"),
            LiteralValue::Text(text) => f.write_str(text),
            LiteralValue::Number(num) => write!(f, "{num}"),
        }
    }
}
