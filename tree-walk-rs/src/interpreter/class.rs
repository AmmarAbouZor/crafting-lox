use std::{collections::HashMap, fmt::Display};

use super::callables::LoxCallable;

#[derive(Debug, Clone, PartialEq)]
pub struct LoxClass {
    name: String,
    // 'LoxCallable is always Lox Function
    // TODO: Check if its better to have a separate struct for LoxFunction
    methods: HashMap<String, LoxCallable>,
}

impl LoxClass {
    pub fn new(name: String, methods: HashMap<String, LoxCallable>) -> Self {
        Self { name, methods }
    }

    pub fn find_method(&self, name: &str) -> Option<&LoxCallable> {
        self.methods.get(name)
    }
}

impl Display for LoxClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}
