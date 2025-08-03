//! Abstract Syntax Tree

use std::fmt::Debug;

use super::LiteralValue;
use crate::Token;

// NOTE: I ported the visitor pattern from the book into Rust pattern matching
// on enums since this what they wanted to achieve.
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Binary {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    Call {
        callee: Box<Expr>,
        paren: Token,
        arguments: Vec<Expr>,
    },
    Get {
        object: Box<Expr>,
        name: Token,
    },
    Grouping {
        expression: Box<Expr>,
    },
    Literal {
        value: LiteralValue,
    },
    Logical {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    Set {
        object: Box<Expr>,
        name: Token,
        value: Box<Expr>,
    },
    Super {
        keyword: Token,
        method: Token,
    },
    This {
        keyword: Token,
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
        value: Box<Expr>,
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
            Expr::Assign {
                name,
                value: expression,
            } => parenthesize("assign", &[expression]),
            Expr::Logical {
                left,
                operator,
                right,
            } => parenthesize(&operator.lexeme, &[left, right]),
            Expr::Call {
                callee,
                paren,
                arguments,
            } => {
                let exprs: Vec<_> = std::iter::once(callee.as_ref())
                    .chain(arguments.iter())
                    .collect();
                parenthesize("function", &exprs)
            }
            Expr::Get { object, name } => parenthesize(format!("Get {name}").as_str(), &[object]),
            Expr::Set {
                object,
                name,
                value,
            } => parenthesize(format!("Set {name}").as_str(), &[object, value]),
            Expr::This { keyword } => String::from("This"),
            Expr::Super { keyword, method } => format!("super.{}", method.lexeme),
        }
    }
}
