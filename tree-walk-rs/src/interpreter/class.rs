use std::{collections::HashMap, fmt::Display};

use crate::{errors::LoxError, interpreter::instance::LoxInstance};

use super::{Interpreter, LoxValue, callables::LoxClassRef, function::LoxFunction};

#[derive(Debug, Clone, PartialEq)]
pub struct LoxClass {
    name: String,
    methods: HashMap<String, LoxFunction>,
    super_class: Option<Box<LoxClassRef>>,
}

impl LoxClass {
    pub fn new(
        name: String,
        methods: HashMap<String, LoxFunction>,
        super_class: Option<LoxClassRef>,
    ) -> Self {
        let super_class = super_class.map(Box::new);
        Self {
            name,
            methods,
            super_class,
        }
    }

    pub fn find_method(&self, name: &str) -> Option<LoxFunction> {
        if let Some(method) = self.methods.get(name) {
            return Some(method.to_owned());
        }

        if let Some(super_c) = self.super_class.as_ref()
            && let Some(method) = super_c.borrow().find_method(name)
        {
            return Some(method);
        }

        None
    }

    pub fn call(
        &self,
        interprerter: &mut Interpreter,
        arguments: &[LoxValue],
    ) -> Result<LoxValue, LoxError> {
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
