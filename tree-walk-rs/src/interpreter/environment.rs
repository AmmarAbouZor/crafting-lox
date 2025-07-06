use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{LoxValue, RuntimeError, Token};

pub type EnvironmentRef = Rc<RefCell<Environment>>;

#[derive(Debug, Default)]
pub struct Environment {
    enclosing: Option<EnvironmentRef>,
    values: HashMap<String, LoxValue>,
}

impl Environment {
    pub fn with_enclosing(enclosing: EnvironmentRef) -> EnvironmentRef {
        let env = Self {
            enclosing: Some(enclosing),
            ..Default::default()
        };

        Rc::new(RefCell::new(env))
    }

    pub fn define(&mut self, key: String, value: LoxValue) {
        self.values.insert(key, value);
    }

    pub fn get(&self, name: &Token) -> Result<LoxValue, RuntimeError> {
        if let Some(val) = self.values.get(&name.lexeme) {
            return Ok(val.to_owned());
        }

        if let Some(enclosing) = &self.enclosing {
            return enclosing.borrow().get(name);
        }

        Err(RuntimeError::new(
            name.to_owned(),
            format!("Undefined variable '{}'.", name.lexeme),
        ))
    }

    pub fn assign(&mut self, name: &Token, value: LoxValue) -> Result<(), RuntimeError> {
        if let Some(old_val) = self.values.get_mut(&name.lexeme) {
            *old_val = value;
            return Ok(());
        };

        if let Some(enclosing) = &self.enclosing {
            return enclosing.borrow_mut().assign(name, value);
        }

        Err(RuntimeError::new(
            name.to_owned(),
            format!("Undefined variable '{}'.", name.lexeme),
        ))
    }
}
