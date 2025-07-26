use std::collections::HashMap;

use crate::{
    Token,
    ast::{Expr, Stmt},
    interpreter::Interpreter,
};

mod error;

pub use error::ResolveError;

type Result<T> = std::result::Result<T, ResolveError>;

#[derive(Debug)]
pub struct Resolver {
    interpreter: Interpreter,
    /// The scope contains the variables in the current scope and
    /// their state with:
    /// - False: Variable declared but not defined (Not initialized with a value)
    /// - True: Variable defined with the initialized value (Which can be nil as well)
    scopes: Vec<HashMap<String, bool>>,
}

impl Resolver {
    pub fn new(interpreter: Interpreter) -> Self {
        Self {
            interpreter,
            scopes: Vec::new(),
        }
    }

    //TODO: Rename if make sense
    pub fn resolve_stmt(&mut self, stmt: &Stmt) -> Result<()> {
        match stmt {
            Stmt::Expression(expr) => {}
            Stmt::Function(func_declaration) => {}
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {}
            Stmt::Print(expr) => {}
            Stmt::Return {
                keyword,
                value_expr,
            } => {}
            Stmt::Var { name, initializer } => return self.resolve_var(name, initializer.as_ref()),
            Stmt::While { condition, body } => {}
            Stmt::Block { statements } => return self.resolve_block(statements),
        }

        Ok(())
    }

    fn resolve_var(&mut self, name: &Token, initializer: Option<&Expr>) -> Result<()> {
        // We need to declare and define a variable in two separated steps because of the
        // the case:
        // ```
        // var a = "outer";
        // {
        //   var a = a;
        // }
        // ```
        // In such case we need to return an error.
        self.declare(name);
        if let Some(expr) = initializer {
            self.resolve_expr(expr)?;
        }
        self.define(name);

        Ok(())
    }

    fn declare(&mut self, name: &Token) {
        if let Some(map) = self.scopes.last_mut() {
            map.insert(name.lexeme.to_owned(), false);
        }
    }

    fn define(&mut self, name: &Token) {
        if let Some(map) = self.scopes.last_mut() {
            let entry = map
                .get_mut(&name.lexeme)
                .expect("Variable must be declared before defining it");
            *entry = true;
        }
    }

    fn resolve_block(&mut self, stmts: &[Stmt]) -> Result<()> {
        self.begin_scope();
        self.resolve_stmts(stmts)?;
        self.end_scope();

        Ok(())
    }

    fn resolve_stmts(&mut self, stmts: &[Stmt]) -> Result<()> {
        for stmt in stmts {
            self.resolve_stmt(stmt)?;
        }

        Ok(())
    }

    pub fn resolve_expr(&mut self, expr: &Expr) -> Result<()> {
        match expr {
            Expr::Binary {
                left,
                operator,
                right,
            } => todo!(),
            Expr::Call {
                callee,
                paren,
                arguments,
            } => todo!(),
            Expr::Grouping { expression } => todo!(),
            Expr::Literal { value } => todo!(),
            Expr::Logical {
                left,
                operator,
                right,
            } => todo!(),
            Expr::Unary { operator, right } => todo!(),
            expr @ Expr::Variable { name } => self.expr_var(expr, name),
            expr @ Expr::Assign { name, value } => self.expr_assign(expr, name, value.as_ref()),
        }
    }

    fn expr_var(&mut self, expr: &Expr, name: &Token) -> Result<()> {
        if let Some(map) = self.scopes.last()
            && map
                .get(&name.lexeme)
                .expect("Variable must be declared before expression resolve")
                == &false
        {
            return Err(ResolveError::new(
                name.to_owned(),
                "Can't read local variable in its own initializer.",
            ));
        }

        self.resolve_local(expr, name);

        Ok(())
    }

    fn expr_assign(&mut self, assign_expr: &Expr, name: &Token, value: &Expr) -> Result<()> {
        self.resolve_expr(value)?;
        self.resolve_local(assign_expr, name);
        Ok(())
    }

    fn resolve_local(&mut self, expr: &Expr, name: &Token) {
        for (idx, map) in self.scopes.iter().enumerate().rev() {
            if map.contains_key(&name.lexeme) {
                //TODO: This feels that would overflow
                self.interpreter.resolve(expr, self.scopes.len() - 1 - idx);
                return;
            }
        }
    }

    fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn end_scope(&mut self) {
        self.scopes.pop();
    }
}
