use std::{collections::HashMap, sync::OnceLock};

type KeywordsMap = HashMap<&'static str, TT>;

use super::token::TokenType as TT;

pub fn get_keywords() -> &'static KeywordsMap {
    static KEYWORDS: OnceLock<KeywordsMap> = OnceLock::new();

    KEYWORDS.get_or_init(|| {
        let items = [
            ("and", TT::And),
            ("class", TT::Class),
            ("else", TT::Else),
            ("false", TT::False),
            ("for", TT::For),
            ("fun", TT::Fun),
            ("if", TT::If),
            ("nil", TT::Nil),
            ("or", TT::Or),
            ("print", TT::Print),
            ("return", TT::Return),
            ("super", TT::Super),
            ("this", TT::This),
            ("true", TT::True),
            ("var", TT::Var),
            ("while", TT::While),
        ];

        HashMap::from_iter(items)
    })
}
