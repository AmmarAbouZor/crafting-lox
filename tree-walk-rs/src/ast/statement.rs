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
    Return {
        keyword: Token,
        value_expr: Option<Expr>,
    },
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
    Class {
        name: Token,
        //NOTE: In the book it says that this is an `Expr::Var`
        // Since `Expr::Var = {name: Token}` I used the name as super_class
        super_class: Option<Token>,
        methods: Vec<FuncDeclaration>,
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
