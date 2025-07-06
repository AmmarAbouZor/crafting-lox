use std::collections::HashMap;

use crate::{LoxValue, RuntimeError, Token};

#[derive(Debug, Default)]
pub struct Environment {
    values: HashMap<String, LoxValue>,
}

impl Environment {
    pub fn define(&mut self, key: String, value: LoxValue) {
        self.values.insert(key, value);
    }

    pub fn get(&mut self, name: &Token) -> Result<&LoxValue, RuntimeError> {
        self.values.get(&name.lexeme).ok_or_else(|| {
            RuntimeError::new(
                name.to_owned(),
                format!("Undefined variable '{}'.", name.lexeme),
            )
        })
    }

    pub fn assign(&mut self, name: &Token, value: LoxValue) -> Result<(), RuntimeError> {
        let old_val = self.values.get_mut(&name.lexeme).ok_or_else(|| {
            RuntimeError::new(
                name.to_owned(),
                format!("Undefined variable '{}'.", name.lexeme),
            )
        })?;

        *old_val = value;

        Ok(())
    }
}
