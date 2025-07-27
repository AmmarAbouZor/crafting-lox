use std::collections::HashMap;

use crate::{
    Token,
    ast::{Expr, FuncDeclaration, Stmt},
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

    pub fn resolve_stmt(&mut self, stmt: &Stmt) -> Result<()> {
        match stmt {
            Stmt::Expression(expr) => self.resolve_expr(expr),
            Stmt::Function(func_declaration) => self.visit_stmt_function(func_declaration),
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                self.resolve_expr(condition)?;
                // Static analyzing resolve both branches, as opposite to interpretation
                // which run one of them only.
                self.resolve_stmt(&then_branch)?;
                if let Some(else_branch) = else_branch {
                    self.resolve_stmt(&else_branch)?;
                }
                Ok(())
            }
            Stmt::Print(expr) => self.resolve_expr(expr),
            Stmt::Return {
                keyword: _,
                value_expr,
            } => {
                if let Some(value) = value_expr {
                    self.resolve_expr(value)?
                }
                Ok(())
            }
            Stmt::Var { name, initializer } => self.resolve_var(name, initializer.as_ref()),
            Stmt::While { condition, body } => {
                self.resolve_expr(condition)?;
                self.resolve_stmt(&body)
            }
            Stmt::Block { statements } => self.resolve_block(statements),
        }
    }

    fn visit_stmt_function(&mut self, func_declaration: &FuncDeclaration) -> Result<()> {
        self.declare(&func_declaration.name);
        self.define(&func_declaration.name);

        self.resolve_function(func_declaration)
    }

    /// Method used for resolving stand-alone function and method on class later.
    fn resolve_function(&mut self, func_declaration: &FuncDeclaration) -> Result<()> {
        self.begin_scope();
        for param in &func_declaration.params {
            self.declare(param);
            self.define(param);
        }
        //TODO: I'm not sure if I need to end scope before return on errors.
        let resolve_res = self.resolve_stmts(&func_declaration.body);
        self.end_scope();

        resolve_res
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
                operator: _,
                right,
            } => {
                self.resolve_expr(&left)?;
                self.resolve_expr(&right)
            }
            Expr::Call {
                callee,
                paren: _,
                arguments,
            } => {
                self.resolve_expr(&callee)?;
                for arg in arguments {
                    self.resolve_expr(arg)?;
                }

                Ok(())
            }
            Expr::Grouping { expression } => self.resolve_expr(&expression),
            Expr::Literal { value: _ } => Ok(()),
            Expr::Logical {
                left,
                operator: _,
                right,
            } => {
                self.resolve_expr(&left)?;
                self.resolve_expr(&right)
            }
            Expr::Unary { operator: _, right } => self.resolve_expr(&right),
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
