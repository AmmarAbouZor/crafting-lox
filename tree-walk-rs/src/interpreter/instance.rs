use std::fmt::Display;

use super::class::LoxClass;

#[derive(Debug, Clone, PartialEq)]
pub struct LoxInstance {
    class: LoxClass,
}

impl LoxInstance {
    pub fn new(class: LoxClass) -> Self {
        Self { class }
    }
}

impl Display for LoxInstance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} instance", self.class)
    }
}
