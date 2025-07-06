use error::RuntimeError;

use crate::{
    Token, TokenType as TT,
    ast::{Expr, Stmt},
};

mod environment;
pub mod error;
mod values;

use environment::Environment;
pub use values::LoxValue;

type Result<T> = std::result::Result<T, RuntimeError>;

#[derive(Debug, Default)]
pub struct Interpreter {
    environment: Environment,
}

impl Interpreter {
    pub fn interpret(&mut self, stmts: &[Stmt]) {
        for stmt in stmts {
            match self.execute(stmt) {
                Ok(()) => {}
                Err(err) => eprintln!("{err}"),
            }
        }
    }

    fn execute(&mut self, stmt: &Stmt) -> Result<()> {
        match stmt {
            Stmt::Expression(expr) => {
                // Expression on their own doesn't need the evaluated
                // value from expression. Examples `1 + 2;` `true;`
                let _ = self.evaluate(expr)?;
            }
            Stmt::Print(expr) => {
                let val = self.evaluate(expr)?;
                println!("{val}");
            }
            Stmt::Var { name, initializer } => {
                let val = if let Some(expr) = initializer {
                    self.evaluate(expr)?
                } else {
                    LoxValue::Nil
                };

                self.environment.define(name.lexeme.to_owned(), val);
            }
        };

        Ok(())
    }

    fn evaluate(&mut self, expr: &Expr) -> Result<LoxValue> {
        match expr {
            Expr::Grouping { expression } => self.evaluate(expression),
            Expr::Literal { value } => Ok(value.into()),
            Expr::Unary { operator, right } => self.evaluate_unary(operator, right),
            Expr::Binary {
                left,
                operator,
                right,
            } => self.evaluate_binary(left, operator, right),
            Expr::Variable { name } => self.environment.get(name).map(|val| val.to_owned()),
        }
    }

    fn evaluate_unary(&mut self, operator: &Token, right: &Expr) -> Result<LoxValue> {
        let right = self.evaluate(right)?;
        let value = match (right, &operator.typ) {
            // Minus
            (LoxValue::Number(num), TT::Minus) => LoxValue::Number(-num),
            (_, TT::Minus) => {
                let err = RuntimeError::new(operator.to_owned(), "Operand must be number.");
                return Err(err);
            }

            // Bang
            (val, TT::Bang) => LoxValue::Boolean(!val.is_truthy()),

            // Unreachable
            (val, oper) => {
                panic!("Unreachable in Unary Expression. Value: {val:?}, Operator: {oper:?}")
            }
        };

        Ok(value)
    }

    fn evaluate_binary(&mut self, left: &Expr, operator: &Token, right: &Expr) -> Result<LoxValue> {
        use LoxValue as V;
        let left = self.evaluate(left)?;
        let right = self.evaluate(right)?;

        let value = match (left, &operator.typ, right) {
            // Arithmetics
            (V::Number(left), TT::Minus, V::Number(right)) => V::Number(left - right),
            (V::Number(left), TT::Slash, V::Number(right)) => V::Number(left / right),
            (V::Number(left), TT::Star, V::Number(right)) => V::Number(left * right),

            // Plus works on numbers and strings
            (V::Number(left), TT::Plus, V::Number(right)) => V::Number(left + right),
            (V::String(left), TT::Plus, V::String(right)) => V::String(format!("{left}{right}")),
            (_, TT::Plus, _) => {
                let err = RuntimeError::new(
                    operator.to_owned(),
                    "Operands must be two numbers or two Strings",
                );
                return Err(err);
            }

            // Comparison
            (V::Number(left), TT::Greater, V::Number(right)) => V::Boolean(left > right),
            (V::Number(left), TT::GreaterEqual, V::Number(right)) => V::Boolean(left >= right),
            (V::Number(left), TT::Less, V::Number(right)) => V::Boolean(left < right),
            (V::Number(left), TT::LessEqual, V::Number(right)) => V::Boolean(left <= right),

            // Error where numeric values and demanded.
            (
                _,
                TT::Minus
                | TT::Slash
                | TT::Star
                | TT::Greater
                | TT::GreaterEqual
                | TT::Less
                | TT::LessEqual,
                _,
            ) => {
                let err = RuntimeError::new(operator.to_owned(), "Operands must be numbers");

                return Err(err);
            }

            // Equality
            (left, TT::EqualEqual, right) => V::Boolean(left == right),
            (left, TT::BangEqual, right) => V::Boolean(left != right),

            // Unreachable
            (l, op, r) => panic!(
                "Unreachable in Binary Expression. Operator: {op:?}, Left: {l:?}, Right: {r:?}"
            ),
        };

        Ok(value)
    }
}
