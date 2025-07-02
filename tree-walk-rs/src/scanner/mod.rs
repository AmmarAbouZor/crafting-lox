mod error;
mod keyword;
mod token;

pub use error::ScanError;
use keyword::get_keywords;
use token::{Token, TokenType as TT};

pub struct Scanner {
    source: String,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: usize,
}

pub struct ScanResults {
    pub tokens: Vec<Token>,
    pub errors: Vec<ScanError>,
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

    pub fn scan_tokens(mut self) -> ScanResults {
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

        ScanResults {
            tokens: self.tokens,
            errors,
        }
    }

    fn scan_intern(&mut self) -> Result<(), ScanError> {
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

            // Comments
            '/' => {
                if self.match_then_advance('/') {
                    // A comment goes until the end of the line.
                    while self.peek() != '\n' && !self.is_at_end() {
                        self.current += 1;
                    }
                } else {
                    self.add_token(TT::Slash);
                }
            }

            // Literals
            '"' => {
                let token = self.parse_string()?;
                self.add_token(token);
            }

            // Empty characters.
            '\n' => self.line += 1,
            ' ' | '\r' | '\t' => {
                // Ignore white spaces
            }

            ch if ch.is_digit(10) => {
                let token = self.parse_number();
                self.add_token(token);
            }

            ch if is_alpha(ch) => {
                let token = self.parse_identifier();
                self.add_token(token);
            }

            _ => return Err(ScanError::new(self.line, "Unexpected Character", None)),
        }

        Ok(())
    }

    #[inline]
    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    /// Reads the next character and advance the current index
    fn advance(&mut self) -> char {
        let ch = self.source.chars().nth(self.current).unwrap();
        self.current += 1;
        ch
    }

    fn peek(&mut self) -> char {
        self.source.chars().nth(self.current).unwrap_or('\0')
    }

    fn peek_next(&mut self) -> char {
        self.source.chars().nth(self.current + 1).unwrap_or('\0')
    }

    fn add_token(&mut self, token_t: TT) {
        let text: String = self.sub_string(self.start, self.current);

        let token = Token::new(token_t, text, self.line);
        self.tokens.push(token);
    }

    fn sub_string(&self, start: usize, end: usize) -> String {
        // This is more safe approach then indexing text directly because of
        // multi-bytes characters. Using `unicode_segmentation` crate is another option.
        self.source.chars().skip(start).take(end - start).collect()
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

    /// Parse string literals until closing quote advancing the current
    /// index to it.
    /// This function assumes that the current character is the one after
    /// the opening quote.
    fn parse_string(&mut self) -> Result<TT, ScanError> {
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1;
            }
            // Advance
            self.current += 1;
        }

        if self.is_at_end() {
            return Err(ScanError::new(self.line, "Unterminated String", None));
        }

        let text = self.sub_string(self.start + 1, self.current);
        // The ending quote
        self.current += 1;

        Ok(TT::String(text))
    }

    /// Parse number until the end advancing the current
    /// index to it.
    fn parse_number(&mut self) -> TT {
        while self.peek().is_digit(10) {
            self.current += 1;
        }

        if self.peek() == '.' && self.peek_next().is_digit(10) {
            // Consume the dot
            self.current += 1;

            while self.peek().is_digit(10) {
                self.current += 1;
            }
        }

        let num_text = self.sub_string(self.start, self.current);
        let num: f64 = num_text.parse().unwrap();

        TT::Number(num)
    }

    /// Parse identifier checking if it's one of the reserved words
    fn parse_identifier(&mut self) -> TT {
        while is_alpha_numeric(self.peek()) {
            self.current += 1;
        }

        let ident = self.sub_string(self.start, self.current);
        get_keywords()
            .get(ident.as_str())
            .map(|tt| tt.to_owned())
            .unwrap_or_else(|| TT::Identifier(ident))
    }
}

/// Check if char is alphabetic or underscore.
fn is_alpha(ch: char) -> bool {
    ch.is_alphabetic() || ch == '_'
}

/// Check if char is alphabetic, underscore or numeric.
fn is_alpha_numeric(ch: char) -> bool {
    is_alpha(ch) || ch.is_digit(10)
}
