use crate::Token;

use super::Expr;

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Expression(Expr),
    Function(FuncDeclaration),
    If {
        condition: Expr,
        then_branch: Box<Stmt>,
        else_branch: Option<Box<Stmt>>,
    },
    Print(Expr),
    Var {
        name: Token,
        initializer: Option<Expr>,
    },
    While {
        condition: Expr,
        body: Box<Stmt>,
    },
    Block {
        statements: Vec<Stmt>,
    },
}

#[derive(Debug, Clone, PartialEq)]
/// Wrapper around function declaration infos to avoid repeat them in
/// both `Stmt` and `LoxCallable`
pub struct FuncDeclaration {
    pub name: Token,
    pub params: Vec<Token>,
    pub body: Vec<Stmt>,
}

impl FuncDeclaration {
    pub fn new(name: Token, params: Vec<Token>, body: Vec<Stmt>) -> Self {
        Self { name, params, body }
    }
}
