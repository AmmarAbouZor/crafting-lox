mod token;

mod error;

pub use error::ScannError;
use token::{Token, TokenType as TT};

pub struct Scanner {
    source: String,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: usize,
}

pub struct ScannResults {
    pub tokens: Vec<Token>,
    pub errors: Vec<ScannError>,
}

impl Scanner {
    pub fn new(source: String) -> Self {
        Self {
            source,
            tokens: Vec::new(),
            start: 0,
            current: 0,
            line: 1,
        }
    }

    pub fn scan_tokens(mut self) -> ScannResults {
        let mut errors = Vec::new();
        while !self.is_at_end() {
            // We are at the beginning of the next lexeme.
            self.start = self.current;
            match self.scan_intern() {
                Ok(()) => {}
                Err(err) => errors.push(err),
            };
        }

        self.tokens.push(Token::new(TT::Eof, "", self.line));

        ScannResults {
            tokens: self.tokens,
            errors,
        }
    }

    fn scan_intern(&mut self) -> Result<(), ScannError> {
        let c = self.advance();
        match c {
            // single character tokens
            '(' => self.add_token(TT::LeftParen),
            ')' => self.add_token(TT::RightParen),
            '{' => self.add_token(TT::LeftBrace),
            '}' => self.add_token(TT::RightBrace),
            ',' => self.add_token(TT::Comma),
            '.' => self.add_token(TT::Dot),
            '-' => self.add_token(TT::Minus),
            '+' => self.add_token(TT::Plus),
            ';' => self.add_token(TT::SemiColon),
            '*' => self.add_token(TT::Star),

            // One or two character tokens
            '!' => {
                if self.match_then_advance('=') {
                    self.add_token(TT::BangEqual);
                } else {
                    self.add_token(TT::Bang);
                }
            }
            '=' => {
                if self.match_then_advance('=') {
                    self.add_token(TT::EqualEqual);
                } else {
                    self.add_token(TT::Equal);
                }
            }
            '<' => {
                if self.match_then_advance('=') {
                    self.add_token(TT::LessEqual);
                } else {
                    self.add_token(TT::Less);
                }
            }
            '>' => {
                if self.match_then_advance('=') {
                    self.add_token(TT::GreaterEqual);
                } else {
                    self.add_token(TT::Greater);
                }
            }
            '/' => {
                // Check for comment case
                if self.match_then_advance('/') {
                    // A comment goes until the end of the line.
                    while self.peek() != '\n' && !self.is_at_end() {
                        self.current += 1;
                    }
                } else {
                    self.add_token(TT::Slash);
                }
            }

            '\n' => self.line += 1,
            ' ' | '\r' | '\t' => {
                // Ignore white spaces
            }

            _ => return Err(ScannError::new(self.line, "Unexpected Character", None)),
        }

        Ok(())
    }

    #[inline]
    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn advance(&mut self) -> char {
        let ch = self.source.chars().nth(self.current).unwrap();
        self.current += 1;
        ch
    }

    fn peek(&mut self) -> char {
        self.source.chars().nth(self.current).unwrap_or('\0')
    }

    fn add_token(&mut self, token_t: TT) {
        // This is more safe approach then indexing text directly because of
        // multi-bytes characters. Using `unicode_segmentation` crate is another option.
        let text: String = self
            .source
            .chars()
            .skip(self.start)
            .take(self.current - self.start)
            .collect();

        let token = Token::new(token_t, text, self.line);
        self.tokens.push(token);
    }

    /// Checks if the next character matches the provided one.
    /// Only then it will consume it.
    fn match_then_advance(&mut self, match_ch: char) -> bool {
        if let Some(char) = self.source.chars().nth(self.current)
            && char == match_ch
        {
            self.current += 1;
            true
        } else {
            false
        }
    }
}
