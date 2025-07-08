use error::ParseError;

use crate::{
    Token, TokenType as TT,
    ast::{Expr, LiteralValue, Stmt},
};

type Result<T> = std::result::Result<T, error::ParseError>;

pub mod error;

const MAX_ARGS_COUNT: usize = 255;

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

    /// Definition:
    /// ```text
    /// declaration → funDecl
    ///             | varDecl
    ///             | statement ;
    /// ```
    fn declaration(&mut self) -> Option<Stmt> {
        let res = if self.match_then_consume(&[TT::Fun]) {
            self.function_declaration("function")
        } else if self.match_then_consume(&[TT::Var]) {
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

    /// Definition:
    /// ```text
    /// funDecl     → "fun" function ;
    /// function    → IDENTIFIER "(" parameters? ")" block ;
    /// parameters  → IDENTIFIER ( "," IDENTIFIER )* ;
    /// ```
    fn function_declaration(&mut self, kind: &str) -> Result<Stmt> {
        // Name:
        let name = self
            .consume_identifier(format!("Expect {kind} name."))?
            .to_owned();

        self.consume(&TT::LeftParen, format!("Expect '(' after {kind} name."))?;

        // Parameters
        let mut params = Vec::new();

        if !self.check(&TT::RightParen) {
            loop {
                if params.len() > MAX_ARGS_COUNT {
                    return Err(ParseError::new(
                        self.peek().to_owned(),
                        format!("Can't have more than {MAX_ARGS_COUNT} arguments."),
                    ));
                }
                let param = self
                    .consume_identifier("Expect parameter name.")?
                    .to_owned();
                params.push(param);

                if !self.match_then_consume(&[TT::Comma]) {
                    break;
                }
            }
        }

        self.consume(&TT::RightParen, "Expect ')' after parameters")?;

        // Body:
        self.consume(&TT::LeftBrace, format!("Expect '{{' before {kind} body."))?;
        let body = self.block()?;

        let stmt = Stmt::Function { name, params, body };

        Ok(stmt)
    }

    fn var_declaration(&mut self) -> Result<Stmt> {
        let name = self.consume_identifier("Expect variable name")?.to_owned();

        let initializer = if self.match_then_consume(&[TT::Equal]) {
            Some(self.expression()?)
        } else {
            None
        };

        self.consume(&TT::SemiColon, "Expect ';' after variable declaration.")?;

        let stmt = Stmt::Var { name, initializer };

        Ok(stmt)
    }

    /// Definition:
    /// ```text
    /// statement → exprStmt
    ///           | forStmt
    ///           | ifStmt
    ///           | printStmt
    ///           | whileStmt
    ///           | block ;
    /// ```
    fn statement(&mut self) -> Result<Stmt> {
        if self.match_then_consume(&[TT::For]) {
            return self.for_statement();
        }
        if self.match_then_consume(&[TT::If]) {
            return self.if_statement();
        }
        if self.match_then_consume(&[TT::Print]) {
            return self.print_statement();
        }

        if self.match_then_consume(&[TT::While]) {
            return self.while_statement();
        }

        if self.match_then_consume(&[TT::LeftBrace]) {
            let statements = self.block()?;
            return Ok(Stmt::Block { statements });
        }

        self.expr_statement()
    }

    /// Definition:
    /// ```text
    /// ifStmt  → "if" "(" expression ")" statement
    ///         ( "else" statement )? ;
    /// ```
    fn if_statement(&mut self) -> Result<Stmt> {
        self.consume(&TT::LeftParen, "Expect '(' after 'if'")?;
        let condition = self.expression()?;
        self.consume(&TT::RightParen, "Expect ')' after condition")?;

        let then_stmt = self.statement()?;
        let then_branch = Box::new(then_stmt);
        let else_branch = if self.match_then_consume(&[TT::Else]) {
            let else_stmt = self.statement()?;
            Some(Box::new(else_stmt))
        } else {
            None
        };

        let stmt = Stmt::If {
            condition,
            then_branch,
            else_branch,
        };

        Ok(stmt)
    }

    /// Definition:
    /// ```text
    /// forStmt → "for" "(" ( varDecl | exprStmt | ";" )
    ///           expression? ";"
    ///           expression? ")" statement ;
    /// ```
    fn for_statement(&mut self) -> Result<Stmt> {
        // NOTE:
        // TWe will desugar the for loop into while loop
        // To be honest: I don't like this solution. I would rather have a clear
        // solution with definitions for easier maintainability.
        // I would rather rewrite this as own statement with own execute function.

        self.consume(&TT::LeftParen, "Expect '(' after for.")?;

        let initializer = if self.match_then_consume(&[TT::SemiColon]) {
            None
        } else if self.match_then_consume(&[TT::Var]) {
            Some(self.var_declaration()?)
        } else {
            Some(self.expr_statement()?)
        };

        let condition = if !self.check(&TT::SemiColon) {
            Some(self.expression()?)
        } else {
            None
        };
        self.consume(&TT::SemiColon, "Expect ';' after loop condition")?;

        let increment = if !self.check(&TT::RightParen) {
            Some(self.expression()?)
        } else {
            None
        };
        self.consume(&TT::RightParen, "Expect ')' after for cluase.")?;

        let mut body = self.statement()?;

        if let Some(increment) = increment {
            body = Stmt::Block {
                statements: vec![body, Stmt::Expression(increment)],
            };
        }

        let condition = condition.unwrap_or(Expr::Literal {
            value: LiteralValue::Boolean(true),
        });

        body = Stmt::While {
            condition,
            body: Box::new(body),
        };

        if let Some(initializer) = initializer {
            body = Stmt::Block {
                statements: vec![initializer, body],
            };
        }

        Ok(body)
    }

    fn block(&mut self) -> Result<Vec<Stmt>> {
        let mut stmts = Vec::new();
        while !self.check(&TT::RightBrace) && !self.at_end() {
            // TODO: Error handling doesn't feel correct here.
            // I need to reconsider when book part is done.
            if let Some(stmt) = self.declaration() {
                stmts.push(stmt);
            }
        }

        self.consume(&TT::RightBrace, "Expect '}' after block.")?;

        Ok(stmts)
    }

    fn print_statement(&mut self) -> Result<Stmt> {
        let expr = self.expression()?;
        self.consume(&TT::SemiColon, "Expect ';' after value.")?;

        let stmt = Stmt::Print(expr);

        Ok(stmt)
    }

    /// Definition:
    /// ```text
    /// whileStmt → "while" "(" expression ")" statement ;
    /// ```
    fn while_statement(&mut self) -> Result<Stmt> {
        self.consume(&TT::LeftParen, "Expect '(' after while.")?;
        let condition = self.expression()?;
        self.consume(&TT::RightParen, "Expect ')' after condition.")?;
        let body = self.statement()?;

        let stmt = Stmt::While {
            condition,
            body: Box::new(body),
        };

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
    ///            | logic_or ;
    /// ```
    fn assignment(&mut self) -> Result<Expr> {
        // L-Value
        let expr = self.or()?;
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

    /// Definition
    /// ```text
    /// logic_or → logic_and ( "or" logic_and )* ;
    /// ```
    fn or(&mut self) -> Result<Expr> {
        let mut expr = self.and()?;

        while self.match_then_consume(&[TT::Or]) {
            let operator = self.previous().to_owned();
            let right = self.and()?;
            expr = Expr::Logical {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    /// Definition
    /// ```text
    /// logic_and → equality ( "and" equality )* ;
    /// ```
    fn and(&mut self) -> Result<Expr> {
        let mut expr = self.equality()?;

        while self.match_then_consume(&[TT::And]) {
            let operator = self.previous().to_owned();
            let right = self.equality()?;
            expr = Expr::Logical {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
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
    ///        | call ;
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
            self.call()
        }
    }

    /// Definition:
    /// ```text
    /// call      → primary ( "(" arguments? ")" )* ;
    /// arguments → expression ( "," expression )* ;
    /// ```
    pub fn call(&mut self) -> Result<Expr> {
        let mut expr = self.primary()?;

        loop {
            if self.match_then_consume(&[TT::LeftParen]) {
                expr = self.finish_call(expr)?;
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn finish_call(&mut self, callee: Expr) -> Result<Expr> {
        let mut arguments = Vec::new();
        if !self.check(&TT::RightParen) {
            loop {
                if arguments.len() >= MAX_ARGS_COUNT {
                    let current_token = self.peek().to_owned();
                    return Err(ParseError::new(
                        current_token,
                        format!("Can't have more than {MAX_ARGS_COUNT} arguments."),
                    ));
                }

                arguments.push(self.expression()?);
                if !self.match_then_consume(&[TT::Comma]) {
                    break;
                }
            }
        }

        let paren = self
            .consume(&TT::RightParen, "Expect ')' after arguments.")?
            .to_owned();

        let expr = Expr::Call {
            callee: Box::new(callee),
            paren,
            arguments,
        };

        Ok(expr)
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

    // Same as consume function but with match because Identifier require
    // checking for matching but not equality.
    fn consume_identifier(&mut self, error_msg: impl Into<String>) -> Result<&Token> {
        let peek = self.peek();
        let ident = match &peek.typ {
            TT::Identifier(..) => self.advance(),
            _ => {
                return Err(ParseError::new(peek.to_owned(), error_msg));
            }
        };

        Ok(ident)
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
