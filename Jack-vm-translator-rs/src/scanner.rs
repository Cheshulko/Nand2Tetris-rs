use std::{borrow::Cow, collections::HashMap};

use once_cell::sync::Lazy;

#[rustfmt::skip] 
static KEYWORDS: Lazy<HashMap<&'static str, TokenType>> = Lazy::new(|| {
    [
        ("push",     TokenType::PUSH),
        ("pop",      TokenType::POP),

        ("add",      TokenType::ADD),
        ("sub",      TokenType::SUB),
        ("neg",      TokenType::NEG),
        ("eq",       TokenType::EQ),
        ("gt",       TokenType::GT),
        ("lt",       TokenType::LT),
        ("and",      TokenType::AND),
        ("or",       TokenType::OR),
        ("not",      TokenType::NOT),

        ("argument", TokenType::ARGUMENT),
        ("local",    TokenType::LOCAL),
        ("static",   TokenType::STATIC),
        ("constant", TokenType::CONSTANT),
        ("this",     TokenType::THIS),
        ("that",     TokenType::THAT),
        ("pointer",  TokenType::POINTER),
        ("temp",     TokenType::TEMP),
        ("label",    TokenType::LABEL),
        ("if-goto",  TokenType::IF_GOTO),
        ("goto",     TokenType::GOTO),
        ("function", TokenType::FUNCTION),
        ("return",   TokenType::RETURN),
        ("call",     TokenType::CALL),
    ]
    .into_iter()
    .collect::<HashMap<&'static str, TokenType>>()
});

#[derive(Debug, Clone)]
#[rustfmt::skip] 
#[allow(non_camel_case_types)]
pub enum TokenType {
    // Literals.
    IDENTIFIER, NUMBER(u16),

    // Keywords:
    PUSH, POP, LABEL, IF_GOTO, GOTO, FUNCTION, RETURN, CALL,

    // Commands.
    ADD, SUB, NEG, EQ, GT, LT, AND, OR, NOT,

    // Segments.
    ARGUMENT, LOCAL, STATIC, CONSTANT,
    THIS, THAT, POINTER, TEMP,

    EOF
}

#[derive(Debug, Clone)]
pub struct Token<'de> {
    pub token_type: TokenType,
    pub lexeme: Cow<'de, str>,
    pub line: usize,
}

impl<'de> Token<'de> {
    pub fn new(token_type: TokenType, lexeme: impl Into<Cow<'de, str>>, line: usize) -> Self {
        Token {
            token_type,
            lexeme: lexeme.into(),
            line,
        }
    }
}

pub struct Scanner<'de> {
    rest: &'de str,
    current: usize,
    line: usize,
    eof: bool,
}

impl<'de> Scanner<'de> {
    pub fn new(source: &'de str) -> Self {
        Self {
            rest: source,
            current: 0,
            line: 1,
            eof: false,
        }
    }

    fn peek_rest_at(&self, pos: usize) -> Option<char> {
        self.rest.chars().nth(pos)
    }

    fn advance_n(&mut self, n: usize) -> &'de str {
        assert!(n >= 1);

        let mut chars = self.rest.chars();
        let mut bytes_n = 0;
        for _ in 0..n {
            let c = chars.next().unwrap();
            bytes_n += c.len_utf8();
        }

        let lexeme = &self.rest[0..bytes_n];
        self.rest = &self.rest[bytes_n..];
        self.current += n;

        lexeme
    }

    fn get_keyword_or_identifier(&self, lemexe: &'de str) -> TokenType {
        KEYWORDS
            .get(lemexe)
            .cloned()
            .unwrap_or(TokenType::IDENTIFIER)
    }

    #[rustfmt::skip]
    fn scan_token(&mut self) -> Option<anyhow::Result<Token<'de>>> {
        fn token<'de>(
            token_type: TokenType,
            lexeme: &'de str,
            line: usize,
        ) -> Option<anyhow::Result<Token<'de>>> {
            Some(Ok(Token::<'de>::new(token_type, lexeme, line)))
        }

        'scan_loop: loop {
            let cur = if let Some(cur) = self.peek_rest_at(0) {
                cur
            } else {
                return None;
            };

            match cur {
                // Meaningless characters.
                ' ' | '\r' | '\t' => {
                    let _ = self.advance_n(1);
                },
                '\n' => {
                    self.line += 1;
                    let _ = self.advance_n(1);
                },
                // Comments
                '/' if self.peek_rest_at(1) == Some('/') => {
                    loop {
                        match self.peek_rest_at(0) {
                            Some(cur) if cur == '\n' => {
                                continue 'scan_loop;
                            }
                            Some(_) => {
                                // Still comment's content
                                let _ = self.advance_n(1);
                            }
                            None => continue 'scan_loop,
                        }
                    }
                },
                // Literals.
                '0'..='9' => {
                    let mut cur_len = 0;

                    fn token_number<'de>(
                        lexeme: &'de str,
                        line: usize,
                    ) -> Option<anyhow::Result<Token<'de>>> {
                        if let Ok(number) = lexeme.parse::<u16>() {
                            token(TokenType::NUMBER(number), lexeme, line)
                        } else {
                            Some(Err(anyhow::anyhow!(format!("[line {line}] Error: Could not parse a number: {lexeme}"))))
                        }                        
                    }

                    loop {
                        match self.peek_rest_at(cur_len) {
                            Some(c) if c.is_digit(10) => {
                                cur_len += 1;
                            }
                            _ => return token_number(self.advance_n(cur_len), self.line),
                        }
                    }
                },
                'a'..='z' | 'A'..='Z' | '-' | '_' | '.' | '$' => {
                    let mut cur_len = 0;

                    loop {
                        match self.peek_rest_at(cur_len) {
                            Some(c) if c.is_alphanumeric() || 
                                c == '-' || c == '_' || c == '.' || c == '$' => {
                                cur_len += 1;
                            }
                            _ => {
                                let lexeme = self.advance_n(cur_len);

                                return token(self.get_keyword_or_identifier(lexeme), lexeme, self.line);
                            }
                        }
                    }
                },
                lexeme => {
                    let _ = self.advance_n(1);
                    let line = self.line;

                    return Some(Err(anyhow::anyhow!(format!("[line {line}] Error: Unexpected character: {lexeme}"))));
                }
            }
        }
    }
}

impl<'de> Iterator for Scanner<'de> {
    type Item = anyhow::Result<Token<'de>>;

    fn next(&mut self) -> Option<Self::Item> {
        let token = self.scan_token();
        if token.is_some() {
            token
        } else {
            if !self.eof {
                self.eof = true;

                Some(Ok(Token::new(TokenType::EOF, "eof", self.line)))
            } else {
                None
            }
        }
    }
}