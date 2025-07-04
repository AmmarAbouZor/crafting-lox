use anyhow::{Result, bail};

use crate::{
    Token, TokenType as TT,
    ast::{Expr, LiteralValue},
};

#[derive(Debug)]
pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    /// Definition: `expression → equality;`
    pub fn expression(&mut self) -> Result<Expr> {
        self.equality()
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
            self.current += 1;
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
    /// ```
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
    /// ```
    /// primary  → NUMBER | STRING | "true" | "false" | "nil"
    ///          | "(" expression ")" ;
    /// ```
    pub fn primary(&mut self) -> Result<Expr> {
        let expr = match &self.advance().typ {
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
                value: LiteralValue::Text(text.to_owned()),
            },
            TT::Number(num) => Expr::Literal {
                value: LiteralValue::Number(*num),
            },
            TT::LeftParen => {
                let expr = self.expression()?;
                self.consume(&TT::RightParen, "Expect ')' after expression.")?;
                Expr::Grouping {
                    expression: Box::new(expr),
                }
            }
            // TODO: Make the error here is correct.
            unexpected => bail!("Unexpected Token in primary clause: {unexpected:?}"),
        };
        Ok(expr)
    }

    fn consume(&mut self, tt: &TT, error_msg: &str) -> Result<Token> {
        todo!()
    }
}
