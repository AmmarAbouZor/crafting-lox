use std::{collections::HashMap, fmt::Display};

use crate::{RuntimeError, Token};

use super::{LoxValue, class::LoxClass};

#[derive(Debug, Clone, PartialEq)]
pub struct LoxInstance {
    class: LoxClass,
    fields: HashMap<String, LoxValue>,
}

impl LoxInstance {
    pub fn new(class: LoxClass) -> Self {
        let fields = HashMap::new();
        Self { class, fields }
    }

    pub fn get(&self, name: &Token) -> Result<LoxValue, RuntimeError> {
        if let Some(value) = self.fields.get(&name.lexeme) {
            return Ok(value.to_owned());
        }

        if let Some(method) = self.class.find_method(&name.lexeme) {
            return Ok(LoxValue::Callable(method.to_owned()));
        }

        Err(RuntimeError::new(
            name.to_owned(),
            format!("Undefined property '{}'.", name.lexeme),
        ))
    }

    pub fn set(&mut self, name: &Token, value: LoxValue) {
        self.fields.insert(name.lexeme.to_owned(), value);
    }
}

impl Display for LoxInstance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} instance", self.class)
    }
}
