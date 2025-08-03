use std::collections::HashMap;

use crate::{
    Token,
    ast::{Expr, FuncDeclaration, Stmt},
    errors::{LoxError, LoxResult},
    interpreter::Interpreter,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FunctionType {
    None,
    Function,
    Initializer,
    Method,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ClassType {
    None,
    Class,
    SubClass,
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

    pub fn resolve_stmts(&mut self, stmts: &[Stmt]) -> LoxResult<()> {
        for stmt in stmts {
            self.resolve_stmt(stmt)?;
        }

        Ok(())
    }

    fn resolve_stmt(&mut self, stmt: &Stmt) -> LoxResult<()> {
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
            } => self.resolve_return(keyword, value_expr.as_ref()),
            Stmt::Var { name, initializer } => self.resolve_var(name, initializer.as_ref()),
            Stmt::While { condition, body } => {
                self.resolve_expr(condition)?;
                self.resolve_stmt(body)
            }
            Stmt::Block { statements } => self.resolve_block(statements),
            Stmt::Class {
                name,
                super_class,
                methods,
            } => self.resolve_stmt_class(name, super_class.as_ref(), methods),
        }
    }

    fn resolve_return(&mut self, keyword: &Token, value_expr: Option<&Expr>) -> LoxResult<()> {
        if self.current_function == FunctionType::None {
            return Err(LoxError::new(
                keyword.to_owned(),
                "Can't return from top level code",
            ));
        }
        if let Some(value) = value_expr {
            if self.current_function == FunctionType::Initializer {
                return Err(LoxError::new(
                    keyword.to_owned(),
                    "Can't return a value fron an initializer.",
                ));
            }
            self.resolve_expr(value)?
        }
        Ok(())
    }

    fn resolve_stmt_class(
        &mut self,
        name: &Token,
        super_class: Option<&Token>,
        methods: &[FuncDeclaration],
    ) -> LoxResult<()> {
        let enclusing_class = self.current_class;
        self.current_class = ClassType::Class;

        let mut s = scopeguard::guard(self, |s| {
            s.current_class = enclusing_class;
        });

        s.declare(name)?;
        s.define(name);

        if let Some(super_class) = super_class {
            // class Foo < Foo {...}
            if name.lexeme == super_class.lexeme {
                return Err(LoxError::new(
                    super_class.to_owned(),
                    "A class can't inherit from itself.",
                ));
            }

            s.current_class = ClassType::SubClass;

            // Resolve
            s.resolve_expr(&Expr::Variable {
                name: super_class.to_owned(),
            })?;

            // Set scope for super
            s.begin_scope();
            s.scopes.last_mut().unwrap().insert("super".into(), true);
        }

        s.begin_scope();
        let mut s = scopeguard::guard(s, |mut s| {
            s.end_scope();
            if super_class.is_some() {
                s.end_scope();
            }
        });

        s.scopes.last_mut().unwrap().insert("this".into(), true);

        for method in methods {
            let declaration = if method.name.lexeme == "init" {
                FunctionType::Initializer
            } else {
                FunctionType::Method
            };
            s.resolve_function(method, declaration)?;
        }

        Ok(())
    }

    fn visit_stmt_function(&mut self, func_declaration: &FuncDeclaration) -> LoxResult<()> {
        self.declare(&func_declaration.name)?;
        self.define(&func_declaration.name);

        self.resolve_function(func_declaration, FunctionType::Function)
    }

    /// Method used for resolving stand-alone function and method on class later.
    fn resolve_function(
        &mut self,
        func_declaration: &FuncDeclaration,
        typ: FunctionType,
    ) -> LoxResult<()> {
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

    fn resolve_var(&mut self, name: &Token, initializer: Option<&Expr>) -> LoxResult<()> {
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

    fn declare(&mut self, name: &Token) -> LoxResult<()> {
        if let Some(map) = self.scopes.last_mut() {
            if map.insert(name.lexeme.to_owned(), false).is_some() {
                return Err(LoxError::new(
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

    fn resolve_block(&mut self, stmts: &[Stmt]) -> LoxResult<()> {
        self.begin_scope();
        let res = self.resolve_stmts(stmts);
        self.end_scope();

        res
    }

    fn resolve_expr(&mut self, expr: &Expr) -> LoxResult<()> {
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
                    return Err(LoxError::new(
                        keyword.to_owned(),
                        "Can't use 'this' outside of a class.",
                    ));
                }
                self.resolve_local(expr, keyword);
                Ok(())
            }
            expr @ Expr::Super { keyword, method: _ } => {
                match self.current_class {
                    ClassType::None => {
                        return Err(LoxError::new(
                            keyword.to_owned(),
                            "Can't use 'super' outside of a class",
                        ));
                    }
                    ClassType::SubClass => {}
                    ClassType::Class => {
                        return Err(LoxError::new(
                            keyword.to_owned(),
                            "Can't use 'super' in a class with no superclass",
                        ));
                    }
                }
                self.resolve_local(expr, keyword);
                Ok(())
            }
        }
    }

    fn expr_var(&mut self, expr: &Expr, name: &Token) -> LoxResult<()> {
        if let Some(map) = self.scopes.last()
            && map.get(&name.lexeme).is_some_and(|val| !val)
        {
            return Err(LoxError::new(
                name.to_owned(),
                "Can't read local variable in its own initializer.",
            ));
        }

        self.resolve_local(expr, name);

        Ok(())
    }

    fn expr_assign(&mut self, assign_expr: &Expr, name: &Token, value: &Expr) -> LoxResult<()> {
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
