use std::cell::Cell;
use std::fmt::Display;

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    // single character tokens
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    SemiColon,
    Slash,
    Star,

    // One or two character tokens
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    // Literals
    Identifier(String),
    String(String),
    Number(f64),

    // Keywords
    And,
    Class,
    Else,
    False,
    Fun,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,

    Eof,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    // NOTE: ID is needed to identify tokens in the same line
    // like `for (var i = 0; i < 20; i = i + 1)`
    id: u64,
    pub typ: TokenType,
    pub lexeme: String,
    pub line: usize,
}

impl Token {
    pub fn new(typ: TokenType, lexeme: impl Into<String>, line: usize) -> Self {
        thread_local! {
            pub static COUNTER: Cell<u64> = const{ Cell::new(0) };
        };

        let id = COUNTER.get();
        COUNTER.set(id + 1);

        Self {
            id,
            typ,
            lexeme: lexeme.into(),
            line,
        }
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.lexeme)
    }
}
