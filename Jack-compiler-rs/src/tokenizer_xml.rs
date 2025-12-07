use once_cell::sync::Lazy;
use serde::{Serialize, Serializer, ser::SerializeStruct};
use std::collections::HashMap;

use crate::{
    Tokens,
    tokenizer::{Keyword, SYMBOLS, Symbol},
};

static SYMBOL_CHARS: Lazy<HashMap<Symbol, char>> = Lazy::new(|| {
    SYMBOLS
        .clone()
        .into_iter()
        .map(|(key, value)| (value, key))
        .collect()
});

impl Serialize for Keyword {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&format!("{:?}", self).to_lowercase())
    }
}

impl Serialize for Symbol {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_char(
            SYMBOL_CHARS
                .get(&self)
                .expect("Always contains an enum key")
                .to_owned(),
        )
    }
}

impl<'de> Serialize for Tokens<'de> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use crate::tokenizer::Constant;
        use crate::tokenizer::TokenType;

        let mut s = serializer.serialize_struct("tokens", self.tokens.len())?;

        for token in &self.tokens {
            match &token.token_type {
                TokenType::Keyword(keyword_type) => s.serialize_field("keyword", keyword_type)?,
                TokenType::Symbol(symbol_type) => s.serialize_field("symbol", symbol_type)?,
                TokenType::Constant(constant_type) => match constant_type {
                    Constant::String(c) => s.serialize_field("stringConstant", c)?,
                    Constant::Integer(i) => s.serialize_field("integerConstant", i)?,
                },
                TokenType::Identifier(_) => s.serialize_field("identifier", &token.lexeme)?,
                TokenType::Eof => {}
            }
        }

        s.end()
    }
}
