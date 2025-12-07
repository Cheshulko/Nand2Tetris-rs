use std::{borrow::Cow, collections::HashMap};

use once_cell::sync::Lazy;

#[rustfmt::skip] 
pub(crate) static KEYWORDS: Lazy<HashMap<&'static str, Keyword>> = Lazy::new(|| {
    [
        ("class",          Keyword::Class),
        ("constructor",    Keyword::Constructor),
        ("function",       Keyword::Function),
        ("method",         Keyword::Method),
        ("field",          Keyword::Field),
        ("static",         Keyword::Static),
        ("var",            Keyword::Var),
        ("int",            Keyword::Int),
        ("char",           Keyword::Char),
        ("boolean",        Keyword::Boolean),
        ("void",           Keyword::Void),
        ("true",           Keyword::True),
        ("false",          Keyword::False),
        ("null",           Keyword::Null),
        ("this",           Keyword::This),
        ("let",            Keyword::Let),
        ("do",             Keyword::Do),
        ("if",             Keyword::If),
        ("else",           Keyword::Else),
        ("while",          Keyword::While),
        ("return",         Keyword::Return)
    ]
    .into_iter()
    .collect::<HashMap<&'static str, Keyword>>()
});

#[rustfmt::skip] 
pub(crate) static SYMBOLS: Lazy<HashMap<char, Symbol>> = Lazy::new(|| {
    [
        ('{',              Symbol::LeftCurlyBrace),
        ('}',              Symbol::RightCurlyBrace),
        ('(',              Symbol::LeftParenthesis),
        (')',              Symbol::RightParenthesis),
        ('[',              Symbol::LeftSquareBracket),
        (']',              Symbol::RightSquareBracket),
        ('.',              Symbol::Dot),
        (',',              Symbol::Comma),
        (';',              Symbol::Semicolon),
        ('+',              Symbol::Plus),
        ('-',              Symbol::Minus),
        ('*',              Symbol::Asterisk),
        ('/',              Symbol::Slash),
        ('&',              Symbol::Ampersand),
        ('|',              Symbol::Pipe),
        ('<',              Symbol::LessThan),
        ('>',              Symbol::GreaterThan),
        ('=',              Symbol::Equal),
        ('~',              Symbol::Tilde),
    ]
    .into_iter()
    .collect::<HashMap<char, Symbol>>()
});

static SYMBOL_LIST: Lazy<Vec<char>> = Lazy::new(|| SYMBOLS.keys().cloned().collect());

#[derive(Debug, Clone)]
pub enum Keyword {
    Class,
    Constructor,
    Function,
    Method,
    Field,
    Static,
    Var,
    Int,
    Char,
    Boolean,
    Void,
    True,
    False,
    Null,
    This,
    Let,
    Do,
    If,
    Else,
    While,
    Return,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum Symbol {
    LeftCurlyBrace,
    RightCurlyBrace,
    LeftParenthesis,
    RightParenthesis,
    LeftSquareBracket,
    RightSquareBracket,
    Dot,
    Comma,
    Semicolon,
    Plus,
    Minus,
    Asterisk,
    Slash,
    Ampersand,
    Pipe,
    LessThan,
    GreaterThan,
    Equal,
    Tilde,
}

#[derive(Debug, Clone)]
pub enum Constant<'de> {
    String(Cow<'de, str>),
    Integer(u16),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Identifier<'de> (pub(super) &'de str);

#[derive(Debug, Clone)]
pub enum TokenType<'de> {
    Keyword(Keyword),
    Symbol(Symbol),
    Constant(Constant<'de>),
    Identifier(Identifier<'de>),

    Eof,
}

#[derive(Debug, Clone)]
pub struct Token<'de> {
    pub token_type: TokenType<'de>,
    pub lexeme: Cow<'de, str>,
    pub _line: usize,
}

impl<'de> Token<'de> {
    pub fn new(token_type: TokenType<'de>, lexeme: impl Into<Cow<'de, str>>, line: usize) -> Self {
        Token {
            token_type,
            lexeme: lexeme.into(),
            _line: line,
        }
    }
}

pub struct Tokenizer<'de> {
    rest: &'de str,
    current: usize,
    line: usize,
    eof: bool,
}

impl<'de> Tokenizer<'de> {
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

    fn get_keyword_or_identifier(&self, lemexe: &'de str) -> TokenType<'de> {
        match KEYWORDS.get(lemexe).cloned() {
            Some(keyword) => TokenType::Keyword(keyword),
            None => {
                TokenType::Identifier(Identifier(lemexe))
            }
        }
    }

    fn get_symbol(&self, symbol: &char) -> TokenType<'static> {
        match SYMBOLS.get(symbol).cloned() {
            Some(symbol) => TokenType::Symbol(symbol),
            _ => panic!(),
        }
    }

    #[rustfmt::skip]
    fn scan_token(&mut self) -> Option<anyhow::Result<Token<'de>>> {
        fn token<'de>(
            token_type: TokenType<'de>,
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
                '/' if self.peek_rest_at(1) == Some('*') => {
                    loop {
                        if self.peek_rest_at(0) == Some('*') && 
                           self.peek_rest_at(1) == Some('/') {
                            self.advance_n(2);
                            break;
                        } else {
                            self.advance_n(1);
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
                            token(TokenType::Constant(Constant::Integer(number)), lexeme, line)
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
                '"' => {
                    let _ = self.advance_n(1);

                    let mut cur_len = 0;
                    loop {
                        match self.peek_rest_at(cur_len) {
                            Some('"') => {
                                let lexeme = self.advance_n(cur_len);
                                let _ = self.advance_n(1);

                                return token(TokenType::Constant(Constant::String(Cow::Borrowed(lexeme))), lexeme, self.line);
                                
                            }
                            None => panic!(),
                            _ => {
                                cur_len += 1;
                            }
                        }
                    }
                }, 
                c if SYMBOL_LIST.contains(&c) => {
                    let lexeme = self.advance_n(1);
                    let x: TokenType<'static> = self.get_symbol(&c);

                    return token(x, lexeme, self.line);
                },
                'a'..='z' | 'A'..='Z' | '-' | '_' | '$' => {
                    let mut cur_len = 0;

                    loop {
                        match self.peek_rest_at(cur_len) {
                            Some(c) if c.is_alphanumeric() ||
                                c == '-' || c == '_' || c == '$' => {
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

impl<'de> Iterator for Tokenizer<'de> {
    type Item = anyhow::Result<Token<'de>>;

    fn next(&mut self) -> Option<Self::Item> {
        let token = self.scan_token();
        if token.is_some() {
            token
        } else {
            if !self.eof {
                self.eof = true;

                Some(Ok(Token::new(TokenType::Eof, "eof", self.line)))
            } else {
                None
            }
        }
    }
}
