//TODO: General: Check why we need the types to implement PartialEq
use std::{cell::RefCell, rc::Rc};

use callables::{CLOCK_NAME, LoxCallable};
use class::LoxClass;
use error::RuntimeError;

use crate::{
    Token, TokenType as TT,
    ast::{Expr, FuncDeclaration, Stmt},
};

mod callables;
mod class;
mod environment;
pub mod error;
mod instance;
mod values;

use environment::{Environment, EnvironmentRef};
pub use values::LoxValue;

type Result<T> = std::result::Result<T, RuntimeError>;

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
        //TODO: Check if the same expression can be defined twice and remove this if not.
        if let Some((_exp, dep)) = self.locals.iter_mut().find(|(ex, _dep)| ex == expr) {
            *dep = depth;
        } else {
            self.locals.push((expr.to_owned(), depth));
        }
    }

    fn execute(&mut self, stmt: &Stmt) -> Result<()> {
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
                let function = LoxCallable::LoxFunction {
                    declaration: declaration.to_owned(),
                    closure: self.environment.clone(),
                };
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
                return Err(RuntimeError::Return {
                    value: Box::new(value),
                });
            }
            Stmt::Class { name, methods } => self.evaluate_class(name, methods)?,
        };

        Ok(())
    }

    fn evaluate_class(&mut self, name: &Token, methods: &[FuncDeclaration]) -> Result<()> {
        self.environment
            .borrow_mut()
            .define(name.lexeme.clone(), LoxValue::Nil);
        let klass = LoxClass::new(name.lexeme.clone());
        self.environment
            .borrow_mut()
            .assign(name, LoxValue::Callable(LoxCallable::Class(klass)))?;

        Ok(())
    }

    fn execute_block(&mut self, statements: &[Stmt], environment: EnvironmentRef) -> Result<()> {
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

    fn evaluate(&mut self, expr: &Expr) -> Result<LoxValue> {
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
        }
    }

    fn evaluate_get(&mut self, object: &Expr, name: &Token) -> Result<LoxValue> {
        match self.evaluate(object)? {
            LoxValue::Instance(lox_instance) => lox_instance.get(name),
            _ => Err(RuntimeError::new(
                name.to_owned(),
                "Only instances have properties.",
            )),
        }
    }

    fn evaluate_set(&mut self, object: &Expr, name: &Token, value: &Expr) -> Result<LoxValue> {
        let object = self.evaluate(object)?;
        let LoxValue::Instance(mut instance) = object else {
            return Err(RuntimeError::new(
                name.to_owned(),
                "Only instances have fields.",
            ));
        };

        let value = self.evaluate(value)?;
        instance.set(name, value.clone());

        Ok(value)
    }

    fn lookup_variable(&mut self, main_expr: &Expr, name: &Token) -> Result<LoxValue> {
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

    fn assign_expr(&mut self, main_exp: &Expr, name: &Token, value: &Expr) -> Result<LoxValue> {
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
    ) -> Result<LoxValue> {
        let callee = self.evaluate(callee)?;
        let mut args = Vec::with_capacity(arguments.len());
        for arg in arguments {
            args.push(self.evaluate(arg)?);
        }

        let callee = match callee {
            LoxValue::Callable(lox_callable) => lox_callable,
            _ => {
                return Err(RuntimeError::new(
                    paren.to_owned(),
                    "Can only call functions and classes.",
                ));
            }
        };

        if callee.arity() != args.len() {
            return Err(RuntimeError::new(
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

    fn evaluate_unary(&mut self, operator: &Token, right: &Expr) -> Result<LoxValue> {
        let right = self.evaluate(right)?;
        let value = match (right, &operator.typ) {
            // Minus
            (LoxValue::Number(num), TT::Minus) => LoxValue::Number(-num),
            (_, TT::Minus) => {
                let err = RuntimeError::new(operator.to_owned(), "Operand must be number.");
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

    fn evaluate_binary(&mut self, left: &Expr, operator: &Token, right: &Expr) -> Result<LoxValue> {
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
                let err = RuntimeError::new(
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
                let err = RuntimeError::new(operator.to_owned(), "Operands must be numbers");

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
    ) -> Result<LoxValue> {
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
