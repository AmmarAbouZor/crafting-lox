use error::ParseError;

use crate::{
    Token, TokenType as TT,
    ast::{Expr, LiteralValue, Stmt},
};

type Result<T> = std::result::Result<T, error::ParseError>;

pub mod error;

#[derive(Debug)]
pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> Result<Vec<Stmt>> {
        let mut stmts = Vec::new();
        while !self.at_end() {
            if let Some(stmt) = self.declaration() {
                stmts.push(stmt)
            }
        }

        Ok(stmts)
    }

    fn declaration(&mut self) -> Option<Stmt> {
        let res = if self.match_then_consume(&[TT::Var]) {
            self.var_declaration()
        } else {
            self.statement()
        };

        //TODO: Error handling here instead of parse function doesn't feel right.
        //However, I'll keep it here to stay in synch with the book for now.
        match res {
            Ok(stmt) => Some(stmt),
            Err(err) => {
                eprintln!("{err}");
                self.synchronize();
                None
            }
        }
    }

    fn var_declaration(&mut self) -> Result<Stmt> {
        let name = if matches!(self.peek().typ, TT::Identifier(..)) {
            self.advance().to_owned()
        } else {
            return Err(ParseError::new(
                self.peek().to_owned(),
                "Expect variable name",
            ));
        };

        let initializer = if self.match_then_consume(&[TT::Equal]) {
            Some(self.expression()?)
        } else {
            None
        };

        self.consume(&TT::SemiColon, "Expect ';' after variable declaration.")?;

        let stmt = Stmt::Var { name, initializer };

        Ok(stmt)
    }

    fn statement(&mut self) -> Result<Stmt> {
        if self.match_then_consume(&[TT::Print]) {
            return self.print_statement();
        }

        self.expr_statement()
    }

    fn print_statement(&mut self) -> Result<Stmt> {
        let expr = self.expression()?;
        self.consume(&TT::SemiColon, "Expect ';' after value.")?;

        let stmt = Stmt::Print(expr);

        Ok(stmt)
    }

    fn expr_statement(&mut self) -> Result<Stmt> {
        let expr = self.expression()?;
        self.consume(&TT::SemiColon, "Expect ';' after expression.")?;

        let stmt = Stmt::Expression(expr);

        Ok(stmt)
    }

    /// Definition: `expression → assignment;`
    pub fn expression(&mut self) -> Result<Expr> {
        self.assignment()
    }

    /// Definition:
    /// ```text
    /// assignment → IDENTIFIER "=" assignment
    ///            | equality ;
    /// ```
    fn assignment(&mut self) -> Result<Expr> {
        // L-Value
        let expr = self.equality()?;
        if self.match_then_consume(&[TT::Equal]) {
            // R-Value
            let value = self.assignment()?;
            match expr {
                Expr::Variable { name } => {
                    return Ok(Expr::Assign {
                        name,
                        expression: Box::new(value),
                    });
                }
                _ => {
                    let equals = self.previous().to_owned();
                    return Err(ParseError::new(equals, "Invalid assignment target."));
                }
            }
        }

        Ok(expr)
    }

    /// Definition: `equality → comparison ( ( "!=" | "==" ) comparison )* ;`
    pub fn equality(&mut self) -> Result<Expr> {
        let mut expr = self.comparison()?;
        while self.match_then_consume(&[TT::BangEqual, TT::EqualEqual]) {
            let operator = self.previous().to_owned();
            let right = self.comparison()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            }
        }

        Ok(expr)
    }

    /// Check if any matches then it consumes the current token.
    fn match_then_consume(&mut self, tts: &[TT]) -> bool {
        if tts.iter().any(|tt| self.check(tt)) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn check(&self, tt: &TT) -> bool {
        &self.peek().typ == tt
    }

    fn advance(&mut self) -> &Token {
        if !self.at_end() {
            self.current += 1;
        }

        self.previous()
    }

    fn at_end(&self) -> bool {
        matches!(self.peek().typ, TT::Eof)
    }

    fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    /// Definition: `comparison → term ( ( ">" | ">=" | "<" | "<=" ) term )*`
    pub fn comparison(&mut self) -> Result<Expr> {
        let mut expr = self.term()?;

        while self.match_then_consume(&[TT::Greater, TT::GreaterEqual, TT::Less, TT::LessEqual]) {
            let operator = self.previous().to_owned();
            let right = self.term()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            }
        }

        Ok(expr)
    }

    /// Definition: `term → factor ( ( "-" | "+" ) factor )*;`
    pub fn term(&mut self) -> Result<Expr> {
        let mut expr = self.factor()?;

        while self.match_then_consume(&[TT::Plus, TT::Minus]) {
            let operator = self.previous().to_owned();
            let right = self.factor()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    /// Definition: `factor → unary ( ( "/" | "*" ) unary )*`
    pub fn factor(&mut self) -> Result<Expr> {
        let mut expr = self.unary()?;

        while self.match_then_consume(&[TT::Slash, TT::Star]) {
            let operator = self.previous().to_owned();
            let right = self.unary()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    /// Definition:
    /// ```text
    /// unary  → ( "!" | "-" ) unary
    ///        | primary ;
    /// ```
    pub fn unary(&mut self) -> Result<Expr> {
        if self.match_then_consume(&[TT::Bang, TT::Minus]) {
            let operator = self.previous().to_owned();
            let right = self.unary()?;
            let expr = Expr::Unary {
                operator,
                right: Box::new(right),
            };

            Ok(expr)
        } else {
            self.primary()
        }
    }

    /// Definition:
    /// ```text
    /// primary  → NUMBER | STRING | "true" | "false" | "nil"
    ///          | "(" expression ")" ;
    /// ```
    pub fn primary(&mut self) -> Result<Expr> {
        let token = self.advance();
        let expr = match token.typ.to_owned() {
            TT::False => Expr::Literal {
                value: LiteralValue::Boolean(false),
            },
            TT::True => Expr::Literal {
                value: LiteralValue::Boolean(true),
            },
            TT::Nil => Expr::Literal {
                value: LiteralValue::Nil,
            },
            TT::String(text) => Expr::Literal {
                value: LiteralValue::Text(text),
            },
            TT::Number(num) => Expr::Literal {
                value: LiteralValue::Number(num),
            },
            TT::LeftParen => {
                let expr = self.expression()?;
                self.consume(&TT::RightParen, "Expect ')' after expression.")?;
                Expr::Grouping {
                    expression: Box::new(expr),
                }
            }
            TT::Identifier(..) => Expr::Variable {
                name: token.to_owned(),
            },
            unexpected => {
                return Err(ParseError::new(
                    self.peek().to_owned(),
                    format!("Expect expression, found {unexpected:?}"),
                ));
            }
        };
        Ok(expr)
    }

    fn consume(&mut self, tt: &TT, error_msg: impl Into<String>) -> Result<&Token> {
        if self.check(tt) {
            Ok(self.advance())
        } else {
            Err(ParseError::new(self.peek().to_owned(), error_msg.into()))
        }
    }

    fn synchronize(&mut self) {
        self.advance();
        while !self.at_end() {
            if self.previous().typ == TT::SemiColon {
                return;
            }
            if matches!(
                self.peek().typ,
                TT::Class
                    | TT::Fun
                    | TT::Var
                    | TT::For
                    | TT::If
                    | TT::While
                    | TT::Print
                    | TT::Return
            ) {
                return;
            }

            self.advance();
        }
    }
}
