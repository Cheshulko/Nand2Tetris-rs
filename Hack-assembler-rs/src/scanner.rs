use std::{borrow::Cow, collections::HashMap};

use once_cell::sync::Lazy;

#[rustfmt::skip] 
static KEYWORDS: Lazy<HashMap<&'static str, TokenType>> = Lazy::new(|| {
    [
        ("M",   TokenType::M),
        ("D",   TokenType::D),
        ("MD",  TokenType::MD),
        ("A",   TokenType::A),
        ("AM",  TokenType::AM),
        ("AD",  TokenType::AD),
        ("AMD", TokenType::AMD),

        ("JGT", TokenType::JGT),
        ("JEQ", TokenType::JEQ),
        ("JGE", TokenType::JGE),
        ("JLT", TokenType::JLT),
        ("JNE", TokenType::JNE),
        ("JLE", TokenType::JLE),
        ("JMP", TokenType::JMP),
    ]
    .into_iter()
    .collect::<HashMap<&'static str, TokenType>>()
});

#[derive(Debug, Clone)]
#[rustfmt::skip] 
#[allow(non_camel_case_types)]
pub enum TokenType {
    // Single-character tokens.
    LEFT_PAREN, RIGHT_PAREN, 
    MINUS, PLUS, EQUAL, 
    BANG, AT, BAR, AMPERSAND, SEMICOLON,

    // Literals.
    IDENTIFIER, NUMBER(u16),

    // Keywords.
    M, D, MD, A, AM, AD, AMD,
    JGT, JEQ, JGE, JLT, JNE, JLE, JMP,

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
                // Single-character tokens.
                '(' => return token(TokenType::LEFT_PAREN,  self.advance_n(1), self.line),
                ')' => return token(TokenType::RIGHT_PAREN, self.advance_n(1), self.line),
                '-' => return token(TokenType::MINUS,       self.advance_n(1), self.line),
                '+' => return token(TokenType::PLUS,        self.advance_n(1), self.line),
                '=' => return token(TokenType::EQUAL,       self.advance_n(1), self.line),
                '!' => return token(TokenType::BANG,        self.advance_n(1), self.line),
                '&' => return token(TokenType::AMPERSAND,   self.advance_n(1), self.line),
                '|' => return token(TokenType::BAR,         self.advance_n(1), self.line),
                '@' => return token(TokenType::AT,          self.advance_n(1), self.line),
                ';' => return token(TokenType::SEMICOLON,   self.advance_n(1), self.line),
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
                'a'..='z' | 'A'..='Z' | '_' | '.' | '$' => {
                    let mut cur_len = 0;

                    loop {
                        match self.peek_rest_at(cur_len) {
                            Some(c) if c.is_alphanumeric() || c == '_' || c == '.' || c == '$' => {
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