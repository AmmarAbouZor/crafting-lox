use std::{fmt::Display, time::SystemTime};

use crate::{RuntimeError, Token, TokenType, ast::FuncDeclaration};

use super::{Interpreter, LoxValue, environment::Environment};

type Result<T> = std::result::Result<T, RuntimeError>;

pub const CLOCK_NAME: &str = "clock";

// TODO: Implementing all the traits isn't necessary if we want to move
// out from LoxValue
#[derive(Debug, Clone, PartialEq)]
pub enum LoxCallable {
    Clock,
    LoxFunction { declaration: FuncDeclaration },
}

impl LoxCallable {
    pub fn call(&self, interprerter: &mut Interpreter, arguments: &[LoxValue]) -> Result<LoxValue> {
        match self {
            LoxCallable::Clock => Self::clock(),
            LoxCallable::LoxFunction { declaration } => {
                Self::call_function(declaration, interprerter, arguments)
            }
        }
    }

    pub fn arity(&self) -> usize {
        match self {
            LoxCallable::Clock => 0,
            LoxCallable::LoxFunction { declaration } => declaration.params.len(),
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

    fn call_function(
        declaration: &FuncDeclaration,
        interprerter: &mut Interpreter,
        arguments: &[LoxValue],
    ) -> Result<LoxValue> {
        let environment = Environment::with_enclosing(interprerter.globals.clone());
        let mut env_borrow = environment.borrow_mut();
        for (arg, param) in arguments.into_iter().zip(declaration.params.iter()) {
            env_borrow.define(param.lexeme.to_owned(), arg.to_owned());
        }
        drop(env_borrow);

        match interprerter.execute_block(&declaration.body, environment) {
            Ok(()) => Ok(LoxValue::Nil),
            Err(RuntimeError::Return { value }) => Ok(value),
            Err(err) => Err(err),
        }
    }
}

impl Display for LoxCallable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoxCallable::Clock => f.write_str("<native fn>"),
            LoxCallable::LoxFunction { declaration } => {
                write!(f, "<fn {}>", declaration.name.lexeme)
            }
        }
    }
}
