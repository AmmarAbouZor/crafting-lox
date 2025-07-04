//! Abstract Syntax Tree

use std::fmt::{Debug, Display};

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
}

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

impl Expr {
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
        }
    }
}

// TODO: Remove after done playing with it.
#[cfg(test)]
mod tests {
    use crate::TokenType;

    use super::*;

    #[test]
    fn test_print() {
        let expr = Expr::Binary {
            left: Box::new(Expr::Unary {
                operator: Token::new(TokenType::Minus, "-", 1),
                right: Box::new(Expr::Literal {
                    value: LiteralValue::Number(123.),
                }),
            }),
            operator: Token::new(TokenType::Star, "*", 1),
            right: Box::new(Expr::Grouping {
                expression: Box::new(Expr::Literal {
                    value: LiteralValue::Number(45.55),
                }),
            }),
        };

        println!("{}", expr.print());
        panic!("Just for printing");
    }
}
