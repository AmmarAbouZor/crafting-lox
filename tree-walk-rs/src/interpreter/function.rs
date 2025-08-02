use std::fmt::Display;

use crate::{RuntimeError, ast::FuncDeclaration};

use super::{
    Interpreter, LoxValue,
    environment::{Environment, EnvironmentRef},
    instance::LoxInstanceRef,
};

#[derive(Debug, Clone, PartialEq)]
pub struct LoxFunction {
    pub declaration: FuncDeclaration,
    pub closure: EnvironmentRef,
}

impl LoxFunction {
    pub fn new(declaration: FuncDeclaration, closure: EnvironmentRef) -> Self {
        Self {
            declaration,
            closure,
        }
    }

    pub fn arity(&self) -> usize {
        self.declaration.params.len()
    }

    pub fn call(
        &self,
        interprerter: &mut Interpreter,
        arguments: &[LoxValue],
    ) -> Result<LoxValue, RuntimeError> {
        let environment = Environment::with_enclosing(self.closure.clone());
        let mut env_borrow = environment.borrow_mut();
        for (arg, param) in arguments.iter().zip(self.declaration.params.iter()) {
            env_borrow.define(param.lexeme.to_owned(), arg.to_owned());
        }
        drop(env_borrow);

        match interprerter.execute_block(&self.declaration.body, environment) {
            Ok(()) => Ok(LoxValue::Nil),
            Err(RuntimeError::Return { value }) => Ok(*value),
            Err(err) => Err(err),
        }
    }

    pub fn bind(&self, instance: LoxInstanceRef) -> LoxFunction {
        let env = Environment::with_enclosing(self.closure.clone());
        env.borrow_mut()
            .define("this".into(), LoxValue::Instance(instance));

        LoxFunction::new(self.declaration.clone(), env)
    }
}

impl Display for LoxFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<fn {}>", self.declaration.name.lexeme)
    }
}
