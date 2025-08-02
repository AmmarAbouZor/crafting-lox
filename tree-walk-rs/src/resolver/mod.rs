use std::collections::HashMap;

use crate::{
    Token,
    ast::{Expr, FuncDeclaration, Stmt},
    interpreter::Interpreter,
};

mod error;

pub use error::ResolveError;

type Result<T> = std::result::Result<T, ResolveError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FunctionType {
    None,
    Function,
    Method,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ClassType {
    None,
    Class,
}

#[derive(Debug)]
pub struct Resolver<'a> {
    interpreter: &'a mut Interpreter,
    /// The scope contains the variables in the current scope and
    /// their state with:
    /// - False: Variable declared but not defined (Not initialized with a value)
    /// - True: Variable defined with the initialized value (Which can be nil as well)
    scopes: Vec<HashMap<String, bool>>,
    current_function: FunctionType,
    current_class: ClassType,
}

impl<'a> Resolver<'a> {
    pub fn new(interpreter: &'a mut Interpreter) -> Self {
        Self {
            interpreter,
            scopes: Vec::new(),
            current_function: FunctionType::None,
            current_class: ClassType::None,
        }
    }

    pub fn resolve_stmts(&mut self, stmts: &[Stmt]) -> Result<()> {
        for stmt in stmts {
            self.resolve_stmt(stmt)?;
        }

        Ok(())
    }

    fn resolve_stmt(&mut self, stmt: &Stmt) -> Result<()> {
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
                self.resolve_stmt(then_branch)?;
                if let Some(else_branch) = else_branch {
                    self.resolve_stmt(else_branch)?;
                }
                Ok(())
            }
            Stmt::Print(expr) => self.resolve_expr(expr),
            Stmt::Return {
                keyword,
                value_expr,
            } => {
                if self.current_function == FunctionType::None {
                    return Err(ResolveError::new(
                        keyword.to_owned(),
                        "Can't return from top level code",
                    ));
                }
                if let Some(value) = value_expr {
                    self.resolve_expr(value)?
                }
                Ok(())
            }
            Stmt::Var { name, initializer } => self.resolve_var(name, initializer.as_ref()),
            Stmt::While { condition, body } => {
                self.resolve_expr(condition)?;
                self.resolve_stmt(body)
            }
            Stmt::Block { statements } => self.resolve_block(statements),
            Stmt::Class { name, methods } => self.resolve_stmt_class(name, methods),
        }
    }

    fn resolve_stmt_class(&mut self, name: &Token, methods: &[FuncDeclaration]) -> Result<()> {
        let enclusing_class = self.current_class;
        self.current_class = ClassType::Class;

        if let Err(err) = self.declare(name) {
            self.current_class = enclusing_class;
            return Err(err);
        }
        self.define(name);

        self.begin_scope();
        let mut s = scopeguard::guard(self, |s| {
            s.end_scope();
            s.current_class = enclusing_class;
        });

        s.scopes.last_mut().unwrap().insert("this".into(), true);

        for method in methods {
            s.resolve_function(method, FunctionType::Method)?;
        }

        Ok(())
    }

    fn visit_stmt_function(&mut self, func_declaration: &FuncDeclaration) -> Result<()> {
        self.declare(&func_declaration.name)?;
        self.define(&func_declaration.name);

        self.resolve_function(func_declaration, FunctionType::Function)
    }

    /// Method used for resolving stand-alone function and method on class later.
    fn resolve_function(
        &mut self,
        func_declaration: &FuncDeclaration,
        typ: FunctionType,
    ) -> Result<()> {
        let enclosing_fun = self.current_function;
        self.current_function = typ;

        self.begin_scope();

        let mut sel = scopeguard::guard(self, |s| {
            s.end_scope();
            s.current_function = enclosing_fun;
        });

        for param in &func_declaration.params {
            sel.declare(param)?;
            sel.define(param);
        }

        sel.resolve_stmts(&func_declaration.body)
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
        self.declare(name)?;
        if let Some(init) = initializer {
            self.resolve_expr(init)?;
        }
        self.define(name);

        Ok(())
    }

    fn declare(&mut self, name: &Token) -> Result<()> {
        if let Some(map) = self.scopes.last_mut() {
            if map.insert(name.lexeme.to_owned(), false).is_some() {
                return Err(ResolveError::new(
                    name.to_owned(),
                    "Already a variable with the same name in this scope",
                ));
            }
        }

        Ok(())
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
        let res = self.resolve_stmts(stmts);
        self.end_scope();

        res
    }

    fn resolve_expr(&mut self, expr: &Expr) -> Result<()> {
        match expr {
            Expr::Binary {
                left,
                operator: _,
                right,
            } => {
                self.resolve_expr(left)?;
                self.resolve_expr(right)
            }
            Expr::Call {
                callee,
                paren: _,
                arguments,
            } => {
                self.resolve_expr(callee)?;
                for arg in arguments {
                    self.resolve_expr(arg)?;
                }

                Ok(())
            }
            Expr::Grouping { expression } => self.resolve_expr(expression),
            Expr::Literal { value: _ } => Ok(()),
            Expr::Logical {
                left,
                operator: _,
                right,
            } => {
                self.resolve_expr(left)?;
                self.resolve_expr(right)
            }
            Expr::Unary { operator: _, right } => self.resolve_expr(right),
            expr @ Expr::Variable { name } => self.expr_var(expr, name),
            expr @ Expr::Assign { name, value } => self.expr_assign(expr, name, value.as_ref()),
            Expr::Get { object, name: _ } => self.resolve_expr(object),
            Expr::Set {
                object,
                name: _,
                value,
            } => {
                self.resolve_expr(object)?;
                self.resolve_expr(value)
            }
            expr @ Expr::This { keyword } => {
                if self.current_class == ClassType::None {
                    return Err(ResolveError::new(
                        keyword.to_owned(),
                        "Can't use 'this' outside of a class.",
                    ));
                }
                self.resolve_local(expr, keyword);
                Ok(())
            }
        }
    }

    fn expr_var(&mut self, expr: &Expr, name: &Token) -> Result<()> {
        if let Some(map) = self.scopes.last()
            && map.get(&name.lexeme).is_some_and(|val| !val)
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
