use std::{collections::HashMap, fmt::Display};

use super::function::LoxFunction;

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
}

impl Display for LoxClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}
