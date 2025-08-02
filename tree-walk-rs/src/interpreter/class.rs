use std::{collections::HashMap, fmt::Display};

use crate::{RuntimeError, interpreter::instance::LoxInstance};

use super::{Interpreter, LoxValue, function::LoxFunction};

#[derive(Debug, Clone, PartialEq)]
pub struct LoxClass {
    name: String,
    methods: HashMap<String, LoxFunction>,
}

impl LoxClass {
    pub fn new(name: String, methods: HashMap<String, LoxFunction>) -> Self {
        Self { name, methods }
    }

    pub fn find_method(&self, name: &str) -> Option<&LoxFunction> {
        self.methods.get(name)
    }

    pub fn call(
        &self,
        interprerter: &mut Interpreter,
        arguments: &[LoxValue],
    ) -> Result<LoxValue, RuntimeError> {
        let instance = LoxInstance::new(self.to_owned());
        if let Some(initializer) = self.find_method("init") {
            initializer
                .bind(instance.clone())
                .call(interprerter, arguments)?;
        };
        Ok(LoxValue::Instance(instance))
    }

    pub fn arity(&self) -> usize {
        if let Some(initializer) = self.find_method("init") {
            initializer.arity()
        } else {
            0
        }
    }
}

impl Display for LoxClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}
