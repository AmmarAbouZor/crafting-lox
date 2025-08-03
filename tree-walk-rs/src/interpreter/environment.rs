use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{LoxValue, RuntimeError, Token};

pub type EnvironmentRef = Rc<RefCell<Environment>>;

#[derive(Debug, Default, PartialEq)]
pub struct Environment {
    pub enclosing: Option<EnvironmentRef>,
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

    pub fn get_at(current: EnvironmentRef, depth: usize, name: &str) -> LoxValue {
        Self::find_ancestor(current, depth)
            .borrow()
            .values
            .get(name)
            .expect(" Value must be avaible since becuase it defined in locals")
            .to_owned()
    }

    fn find_ancestor(current: EnvironmentRef, distance: usize) -> EnvironmentRef {
        let mut env = current;
        for _ in 0..distance {
            let temp = env
                .borrow()
                .enclosing
                .as_ref()
                .expect("It should contain a value")
                .clone();
            env = temp
        }

        env
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

    pub fn assign_at(current: EnvironmentRef, distance: usize, name: &Token, value: LoxValue) {
        Self::find_ancestor(current, distance)
            .borrow_mut()
            .values
            .insert(name.lexeme.to_owned(), value);
    }
}
