use std::fmt::Display;

use crate::RuntimeError;

use super::{Interpreter, LoxValue};

type Result<T> = std::result::Result<T, RuntimeError>;

// TODO: Implementing all the traits isn't necessary if we want to move
// out from LoxValue
#[derive(Debug, Clone, PartialEq)]
pub enum LoxCallable {}

impl LoxCallable {
    pub fn call(&self, interprerter: &mut Interpreter, arguments: &[LoxValue]) -> Result<LoxValue> {
        todo!()
    }

    pub fn arity(&self) -> usize {
        todo!()
    }
}

impl Display for LoxCallable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        //TODO: For now. This only needed because LoxCallable is currently
        //part of LoxValue
        write!(f, "{self:?}")
    }
}
