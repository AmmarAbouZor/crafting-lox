use std::{cell::RefCell, collections::HashMap, rc::Rc};

use callables::{CLOCK_NAME, LoxCallable};
use class::LoxClass;
use function::LoxFunction;
use instance::LoxInstance;

use crate::{
    Token, TokenType as TT,
    ast::{Expr, FuncDeclaration, Stmt},
    errors::{LoxError, LoxResult},
};

mod callables;
mod class;
mod environment;
mod function;
mod instance;
mod values;

use environment::{Environment, EnvironmentRef};
pub use values::LoxValue;

#[derive(Debug)]
pub struct Interpreter {
    globals: EnvironmentRef,
    environment: EnvironmentRef,
    // This is implemented as a map in the book, however this is not possible
    // in rust because `Expr` can't implement `Ord, Eq, Hash`
    locals: Vec<(Expr, usize)>,
}

impl Default for Interpreter {
    fn default() -> Self {
        Self::new()
    }
}

impl Interpreter {
    pub fn new() -> Self {
        let mut globals = Environment::default();
        globals.define(CLOCK_NAME.into(), LoxValue::Callable(LoxCallable::Clock));
        let globals = Rc::new(RefCell::new(globals));
        let environment = globals.clone();

        Self {
            globals,
            environment,
            locals: Vec::new(),
        }
    }
    pub fn interpret(&mut self, stmts: &[Stmt]) {
        for stmt in stmts {
            match self.execute(stmt) {
                Ok(()) => {}
                Err(err) => eprintln!("{err}"),
            }
        }
    }

    pub fn resolve(&mut self, expr: &Expr, depth: usize) {
        if let Some((_exp, dep)) = self.locals.iter_mut().find(|(ex, _dep)| ex == expr) {
            *dep = depth;
        } else {
            self.locals.push((expr.to_owned(), depth));
        }
    }

    fn execute(&mut self, stmt: &Stmt) -> LoxResult<()> {
        match stmt {
            Stmt::Expression(expr) => {
                // Expression on their own doesn't need the evaluated
                // value from expression. Examples `1 + 2;` `true;`
                let _ = self.evaluate(expr)?;
            }
            Stmt::Print(expr) => {
                let val = self.evaluate(expr)?;
                println!("{val}");
            }
            Stmt::Var { name, initializer } => {
                let val = if let Some(expr) = initializer {
                    self.evaluate(expr)?
                } else {
                    LoxValue::Nil
                };

                self.environment
                    .borrow_mut()
                    .define(name.lexeme.to_owned(), val);
            }
            Stmt::Block { statements } => {
                let env = Environment::with_enclosing(self.environment.clone());
                self.execute_block(statements, env)?;
            }
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let cond_val = self.evaluate(condition)?;
                if cond_val.is_truthy() {
                    self.execute(then_branch)?;
                } else if let Some(else_branch) = else_branch {
                    self.execute(else_branch)?;
                }
            }
            Stmt::While { condition, body } => {
                while self.evaluate(condition)?.is_truthy() {
                    self.execute(body)?;
                }
            }
            Stmt::Function(declaration) => {
                let func =
                    LoxFunction::new(declaration.to_owned(), self.environment.clone(), false);
                let function = LoxCallable::LoxFunction(func);
                self.environment.borrow_mut().define(
                    declaration.name.lexeme.to_owned(),
                    LoxValue::Callable(function),
                );
            }
            Stmt::Return {
                keyword: _,
                value_expr,
            } => {
                let value = match value_expr {
                    Some(expr) => self.evaluate(expr)?,
                    None => LoxValue::Nil,
                };

                // Misuse of errors since they will bubble up the call stack
                return Err(LoxError::Return {
                    value: Box::new(value),
                });
            }
            Stmt::Class {
                name,
                super_class,
                methods,
            } => self.evaluate_class(name, super_class.as_ref(), methods)?,
        };

        Ok(())
    }

    fn evaluate_class(
        &mut self,
        name: &Token,
        super_class: Option<&Token>,
        methods: &[FuncDeclaration],
    ) -> LoxResult<()> {
        let super_class = if let Some(super_class) = super_class {
            let class = self.evaluate(&Expr::Variable {
                name: super_class.to_owned(),
            })?;
            match class {
                LoxValue::Callable(LoxCallable::Class(class)) => Some(class),
                _ => {
                    return Err(LoxError::new(
                        super_class.to_owned(),
                        "Superclass must be a class.",
                    ));
                }
            }
        } else {
            None
        };

        self.environment
            .borrow_mut()
            .define(name.lexeme.clone(), LoxValue::Nil);

        if let Some(super_class) = super_class.clone() {
            self.environment = Environment::with_enclosing(self.environment.clone());
            self.environment.borrow_mut().define(
                "super".into(),
                LoxValue::Callable(LoxCallable::Class(super_class)),
            );
        }

        let has_super = super_class.is_some();
        let s = scopeguard::guard(self, |s| {
            if has_super {
                let enclosing = s.environment.borrow().enclosing.as_ref().unwrap().clone();
                s.environment = enclosing;
            }
        });

        let mut meth = HashMap::new();

        for method in methods {
            let is_initializer = method.name.lexeme == "init";
            let function =
                LoxFunction::new(method.to_owned(), s.environment.clone(), is_initializer);
            meth.insert(method.name.lexeme.to_owned(), function);
        }

        let klass = LoxClass::new(name.lexeme.clone(), meth, super_class);
        let klass = Rc::new(RefCell::new(klass));
        s.environment
            .borrow_mut()
            .assign(name, LoxValue::Callable(LoxCallable::Class(klass)))?;

        Ok(())
    }

    fn execute_block(&mut self, statements: &[Stmt], environment: EnvironmentRef) -> LoxResult<()> {
        let prev_env = self.environment.clone();

        self.environment = environment;

        let mut sel = scopeguard::guard(self, |s| {
            s.environment = prev_env;
        });

        for stmt in statements {
            sel.execute(stmt)?;
        }

        Ok(())
    }

    fn evaluate(&mut self, expr: &Expr) -> LoxResult<LoxValue> {
        match expr {
            Expr::Grouping { expression } => self.evaluate(expression),
            Expr::Literal { value } => Ok(value.into()),
            Expr::Unary { operator, right } => self.evaluate_unary(operator, right),
            Expr::Binary {
                left,
                operator,
                right,
            } => self.evaluate_binary(left, operator, right),
            expr @ Expr::Variable { name } => self.lookup_variable(expr, name),
            expr @ Expr::Assign { name, value } => self.assign_expr(expr, name, value),
            Expr::Logical {
                left,
                operator,
                right,
            } => self.evaluate_logical(left, operator, right),
            Expr::Call {
                callee,
                paren,
                arguments,
            } => self.evaluate_call(callee, paren, arguments),
            Expr::Get { object, name } => self.evaluate_get(object, name),
            Expr::Set {
                object,
                name,
                value,
            } => self.evaluate_set(object, name, value),
            expr @ Expr::This { keyword } => self.lookup_variable(expr, keyword),
            expr @ Expr::Super { keyword: _, method } => self.evaluate_super(expr, method),
        }
    }

    fn evaluate_super(&mut self, expr: &Expr, method: &Token) -> LoxResult<LoxValue> {
        let distance = self
            .get_distance(expr)
            .expect("Superclass is registered in resolver");

        let super_value = Environment::get_at(self.environment.clone(), distance, "super");
        let super_class = match &super_value {
            LoxValue::Callable(LoxCallable::Class(klass)) => klass,
            _ => panic!("We must get class when asking fro 'super'"),
        };

        let this_instance = Environment::get_at(self.environment.clone(), distance - 1, "this");
        let this_instance = match this_instance {
            LoxValue::Instance(inst) => inst,
            _ => panic!("We must get instance when asking for 'this'"),
        };

        let method = super_class
            .borrow()
            .find_method(&method.lexeme)
            .ok_or_else(|| {
                LoxError::new(
                    method.to_owned(),
                    format!("Undefined property '{}'.", method.lexeme),
                )
            })?;

        let method = method.bind(this_instance);
        Ok(LoxValue::Callable(LoxCallable::LoxFunction(method)))
    }

    fn evaluate_get(&mut self, object: &Expr, name: &Token) -> LoxResult<LoxValue> {
        match self.evaluate(object)? {
            LoxValue::Instance(lox_instance) => LoxInstance::get(lox_instance, name),
            _ => Err(LoxError::new(
                name.to_owned(),
                "Only instances have properties.",
            )),
        }
    }

    fn evaluate_set(&mut self, object: &Expr, name: &Token, value: &Expr) -> LoxResult<LoxValue> {
        let object = self.evaluate(object)?;
        let LoxValue::Instance(instance) = object else {
            return Err(LoxError::new(
                name.to_owned(),
                "Only instances have fields.",
            ));
        };

        let value = self.evaluate(value)?;
        instance.borrow_mut().set(name, value.clone());

        Ok(value)
    }

    fn lookup_variable(&mut self, main_expr: &Expr, name: &Token) -> LoxResult<LoxValue> {
        let distance = self.get_distance(main_expr);
        if let Some(dist) = distance {
            let val = Environment::get_at(self.environment.clone(), dist, &name.lexeme);
            Ok(val)
        } else {
            self.globals.borrow().get(name)
        }
    }

    fn get_distance(&self, expr: &Expr) -> Option<usize> {
        self.locals
            .iter()
            .find(|(ex, _depth)| ex == expr)
            .map(|(_, depth)| *depth)
    }

    fn assign_expr(&mut self, main_exp: &Expr, name: &Token, value: &Expr) -> LoxResult<LoxValue> {
        let value = self.evaluate(value)?;
        let dist = self.get_distance(main_exp);
        if let Some(distance) = dist {
            Environment::assign_at(self.environment.clone(), distance, name, value.clone());
        } else {
            self.globals.borrow_mut().assign(name, value.clone())?;
        }

        Ok(value)
    }

    fn evaluate_call(
        &mut self,
        callee: &Expr,
        paren: &Token,
        arguments: &[Expr],
    ) -> LoxResult<LoxValue> {
        let callee = self.evaluate(callee)?;
        let mut args = Vec::with_capacity(arguments.len());
        for arg in arguments {
            args.push(self.evaluate(arg)?);
        }

        let callee = match callee {
            LoxValue::Callable(lox_callable) => lox_callable,
            _ => {
                return Err(LoxError::new(
                    paren.to_owned(),
                    "Can only call functions and classes.",
                ));
            }
        };

        if callee.arity() != args.len() {
            return Err(LoxError::new(
                paren.to_owned(),
                format!(
                    "Expected {} arguments but got {}.",
                    callee.arity(),
                    args.len()
                ),
            ));
        }

        callee.call(self, &args)
    }

    fn evaluate_unary(&mut self, operator: &Token, right: &Expr) -> LoxResult<LoxValue> {
        let right = self.evaluate(right)?;
        let value = match (right, &operator.typ) {
            // Minus
            (LoxValue::Number(num), TT::Minus) => LoxValue::Number(-num),
            (_, TT::Minus) => {
                let err = LoxError::new(operator.to_owned(), "Operand must be number.");
                return Err(err);
            }

            // Bang
            (val, TT::Bang) => LoxValue::Boolean(!val.is_truthy()),

            // Unreachable
            (val, oper) => {
                panic!("Unreachable in Unary Expression. Value: {val:?}, Operator: {oper:?}")
            }
        };

        Ok(value)
    }

    fn evaluate_binary(
        &mut self,
        left: &Expr,
        operator: &Token,
        right: &Expr,
    ) -> LoxResult<LoxValue> {
        use LoxValue as V;
        let left = self.evaluate(left)?;
        let right = self.evaluate(right)?;

        let value = match (left, &operator.typ, right) {
            // Arithmetics
            (V::Number(left), TT::Minus, V::Number(right)) => V::Number(left - right),
            (V::Number(left), TT::Slash, V::Number(right)) => V::Number(left / right),
            (V::Number(left), TT::Star, V::Number(right)) => V::Number(left * right),

            // Plus works on numbers and strings
            (V::Number(left), TT::Plus, V::Number(right)) => V::Number(left + right),
            (V::String(left), TT::Plus, V::String(right)) => V::String(format!("{left}{right}")),
            (_, TT::Plus, _) => {
                let err = LoxError::new(
                    operator.to_owned(),
                    "Operands must be two numbers or two Strings",
                );
                return Err(err);
            }

            // Comparison
            (V::Number(left), TT::Greater, V::Number(right)) => V::Boolean(left > right),
            (V::Number(left), TT::GreaterEqual, V::Number(right)) => V::Boolean(left >= right),
            (V::Number(left), TT::Less, V::Number(right)) => V::Boolean(left < right),
            (V::Number(left), TT::LessEqual, V::Number(right)) => V::Boolean(left <= right),

            // Error where numeric values and demanded.
            (
                _,
                TT::Minus
                | TT::Slash
                | TT::Star
                | TT::Greater
                | TT::GreaterEqual
                | TT::Less
                | TT::LessEqual,
                _,
            ) => {
                let err = LoxError::new(operator.to_owned(), "Operands must be numbers");

                return Err(err);
            }

            // Equality
            (left, TT::EqualEqual, right) => V::Boolean(left == right),
            (left, TT::BangEqual, right) => V::Boolean(left != right),

            // Unreachable
            (l, op, r) => panic!(
                "Unreachable in Binary Expression. Operator: {op:?}, Left: {l:?}, Right: {r:?}"
            ),
        };

        Ok(value)
    }

    fn evaluate_logical(
        &mut self,
        left: &Expr,
        operator: &Token,
        right: &Expr,
    ) -> LoxResult<LoxValue> {
        // Evaluate left first and only execute right if logical expand to it.
        // This is necessary to avoid any side effect from executing right.

        let left = self.evaluate(left)?;
        match &operator.typ {
            TT::Or => {
                if left.is_truthy() {
                    return Ok(left);
                }
            }
            TT::And => {
                if !left.is_truthy() {
                    return Ok(left);
                }
            }
            invalid => panic!("Invalid logical operator: {invalid:?}"),
        }

        self.evaluate(right)
    }
}
