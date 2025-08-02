use std::{cell::RefCell, collections::HashMap, fmt::Display, rc::Rc};

use crate::{RuntimeError, Token};

use super::{LoxValue, callables::LoxCallable, class::LoxClass};

pub type LoxInstanceRef = Rc<RefCell<LoxInstance>>;

#[derive(Debug, Clone, PartialEq)]
pub struct LoxInstance {
    class: LoxClass,
    fields: HashMap<String, LoxValue>,
}

impl LoxInstance {
    pub fn new(class: LoxClass) -> LoxInstanceRef {
        let fields = HashMap::new();
        let instance = Self { class, fields };
        Rc::new(RefCell::new(instance))
    }

    pub fn get(inst_ref: LoxInstanceRef, name: &Token) -> Result<LoxValue, RuntimeError> {
        let instance = inst_ref.borrow();
        if let Some(value) = instance.fields.get(&name.lexeme) {
            return Ok(value.to_owned());
        }

        if let Some(method) = instance.class.find_method(&name.lexeme) {
            let func = method.bind(inst_ref.clone());
            return Ok(LoxValue::Callable(LoxCallable::LoxFunction(func)));
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
