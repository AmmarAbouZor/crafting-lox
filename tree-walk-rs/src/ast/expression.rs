//! Abstract Syntax Tree

use std::fmt::Debug;

use super::LiteralValue;
use crate::Token;

// NOTE: I ported the visitor pattern from the book into Rust pattern matching
// on enums since this what they wanted to achieve.
#[derive(Debug, Clone)]
pub enum Expr {
    Binary {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    Grouping {
        expression: Box<Expr>,
    },
    Literal {
        value: LiteralValue,
    },
    Unary {
        operator: Token,
        right: Box<Expr>,
    },
    Variable {
        name: Token,
    },
    Assign {
        name: Token,
        expression: Box<Expr>,
    },
}

impl Expr {
    /// Method is for debugging purpose only.
    #[allow(unused)]
    pub fn print(&self) -> String {
        fn parenthesize(name: &str, exprs: &[&Expr]) -> String {
            let mut text = format!("({name}");
            for expr in exprs {
                text.push(' ');
                text.push_str(&expr.print());
            }
            text.push(')');

            text
        }

        match self {
            Expr::Binary {
                left,
                operator,
                right,
            } => parenthesize(&operator.lexeme, &[left, right]),
            Expr::Grouping { expression } => parenthesize("group", &[expression]),
            Expr::Literal { value } => value.to_string(),
            Expr::Unary { operator, right } => parenthesize(&operator.lexeme, &[right]),
            Expr::Variable { name } => format!("Variable: {name}"),
            Expr::Assign { name, expression } => parenthesize("assign", &[expression]),
        }
    }
}
