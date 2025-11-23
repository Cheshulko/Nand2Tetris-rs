use std::borrow::Cow;
use std::iter::Peekable;

use crate::scanner::{Token, TokenType};

macro_rules! consume {
    ($tokens:expr) => {
        $tokens.next().ok_or(anyhow::anyhow!(
            "Could not consume a token. Token list is empty"
        ))
    };
}

macro_rules! consume_number {
    ($tokens:expr) => {
        match consume!($tokens)? {
            Token {
                token_type: TokenType::NUMBER(value),
                ..
            } => anyhow::Result::<u16>::Ok(value),
            token => {
                anyhow::bail!("Unexpected token. Expected NUMBER but got {:?}", token)
            }
        }
    };
}

macro_rules! consume_identifier {
    ($tokens:expr) => {
        match consume!($tokens)? {
            Token {
                token_type: TokenType::IDENTIFIER,
                lexeme: lemexe,
                ..
            } => anyhow::Result::<Cow<'_, str>>::Ok(lemexe),
            token => {
                anyhow::bail!("Unexpected token. Expected IDENTIFIER but got {:?}", token)
            }
        }
    };
}

macro_rules! consume_and_ensure_matches {
    ($tokens:expr, $( $pattern:pat ),* $(,)?) => {
        match $tokens.next() {
            $(Some(token @ Token {
                token_type: $pattern,
                ..
            }) => anyhow::Result::<Token>::Ok(token), )*
            token => {
                let expected_patterns = vec![$(stringify!($pattern)),*];
                anyhow::bail!(
                    "Unexpected token. Expected one of: {} but got {:?}",
                    expected_patterns.join(", "),
                    token
                )
            },
        }
    };
}

#[derive(Debug)]
pub enum Segment {
    Argument { offset: u16 },
    Local { offset: u16 },
    Static { offset: u16 },
    Constant { value: u16 },
    This { offset: u16 },
    That { offset: u16 },
    Pointer { offset: u16 },
    Temp { offset: u16 },
}

#[derive(Debug)]
pub enum Node<'de> {
    Push { segment: Segment },
    Pop { segment: Segment },
    Label { name: Cow<'de, str> },
    IfGoto { name: Cow<'de, str> },
    Goto { name: Cow<'de, str> },
    Function { name: Cow<'de, str>, n_locals: u16 },
    Call { name: Cow<'de, str>, n_args: u16 },
    Return,
    Add,
    Sub,
    Neg,
    Eq,
    Gt,
    Lt,
    And,
    Or,
    Not,
}

pub struct Parser<'de, I: Iterator<Item = Token<'de>>> {
    tokens: Peekable<I>,
}

impl<'de, I> Parser<'de, I>
where
    I: Iterator<Item = Token<'de>>,
{
    pub fn new(tokens: I) -> Parser<'de, I> {
        Parser {
            tokens: tokens.peekable(),
        }
    }

    pub fn parse(&mut self) -> Option<anyhow::Result<Node<'de>>> {
        while let Some(token) = self.tokens.peek() {
            if matches!(token.token_type, TokenType::EOF) {
                return None;
            }

            if matches!(token.token_type, TokenType::PUSH) {
                return Some(self.parse_push());
            }
            if matches!(token.token_type, TokenType::POP) {
                return Some(self.parse_pop());
            }
            if matches!(token.token_type, TokenType::LABEL) {
                return Some(self.parse_label());
            }
            if matches!(token.token_type, TokenType::IF_GOTO) {
                return Some(self.parse_if_goto());
            }
            if matches!(token.token_type, TokenType::GOTO) {
                return Some(self.parse_goto());
            }
            if matches!(token.token_type, TokenType::FUNCTION) {
                return Some(self.parse_function());
            }
            if matches!(token.token_type, TokenType::RETURN) {
                return Some(self.parse_return());
            }
            if matches!(token.token_type, TokenType::CALL) {
                return Some(self.parse_call());
            }
            if matches!(token.token_type, TokenType::ADD) {
                return Some(self.parse_add());
            }
            if matches!(token.token_type, TokenType::SUB) {
                return Some(self.parse_sub());
            }
            if matches!(token.token_type, TokenType::NEG) {
                return Some(self.parse_neg());
            }
            if matches!(token.token_type, TokenType::EQ) {
                return Some(self.parse_eq());
            }
            if matches!(token.token_type, TokenType::GT) {
                return Some(self.parse_gt());
            }
            if matches!(token.token_type, TokenType::LT) {
                return Some(self.parse_lt());
            }
            if matches!(token.token_type, TokenType::AND) {
                return Some(self.parse_and());
            }
            if matches!(token.token_type, TokenType::OR) {
                return Some(self.parse_or());
            }
            if matches!(token.token_type, TokenType::NOT) {
                return Some(self.parse_not());
            }

            return None;
        }

        unreachable!()
    }

    fn parse_push(&mut self) -> anyhow::Result<Node<'de>> {
        let _ = consume_and_ensure_matches!(self.tokens, TokenType::PUSH)?;
        let segment = self.parse_segment()?;

        Ok(Node::Push { segment })
    }

    fn parse_pop(&mut self) -> anyhow::Result<Node<'de>> {
        let _ = consume_and_ensure_matches!(self.tokens, TokenType::POP)?;
        let segment = self.parse_segment()?;

        Ok(Node::Pop { segment })
    }

    fn parse_label(&mut self) -> anyhow::Result<Node<'de>> {
        let _ = consume_and_ensure_matches!(self.tokens, TokenType::LABEL)?;
        let name = consume_identifier!(self.tokens)?;

        Ok(Node::Label { name })
    }

    fn parse_if_goto(&mut self) -> anyhow::Result<Node<'de>> {
        let _ = consume_and_ensure_matches!(self.tokens, TokenType::IF_GOTO)?;
        let name = consume_identifier!(self.tokens)?;

        Ok(Node::IfGoto { name })
    }

    fn parse_goto(&mut self) -> anyhow::Result<Node<'de>> {
        let _ = consume_and_ensure_matches!(self.tokens, TokenType::GOTO)?;
        let name = consume_identifier!(self.tokens)?;

        Ok(Node::Goto { name })
    }

    fn parse_function(&mut self) -> anyhow::Result<Node<'de>> {
        let _ = consume_and_ensure_matches!(self.tokens, TokenType::FUNCTION)?;
        let name = consume_identifier!(self.tokens)?;
        let n_locals = consume_number!(self.tokens)?;

        Ok(Node::Function { name, n_locals })
    }

    fn parse_return(&mut self) -> anyhow::Result<Node<'de>> {
        let _ = consume_and_ensure_matches!(self.tokens, TokenType::RETURN)?;

        Ok(Node::Return)
    }
    fn parse_call(&mut self) -> anyhow::Result<Node<'de>> {
        let _ = consume_and_ensure_matches!(self.tokens, TokenType::CALL)?;
        let name = consume_identifier!(self.tokens)?;
        let n_args = consume_number!(self.tokens)?;

        Ok(Node::Call { name, n_args })
    }

    fn parse_add(&mut self) -> anyhow::Result<Node<'de>> {
        let _ = consume_and_ensure_matches!(self.tokens, TokenType::ADD)?;

        Ok(Node::Add)
    }

    fn parse_sub(&mut self) -> anyhow::Result<Node<'de>> {
        let _ = consume_and_ensure_matches!(self.tokens, TokenType::SUB)?;

        Ok(Node::Sub)
    }

    fn parse_neg(&mut self) -> anyhow::Result<Node<'de>> {
        let _ = consume_and_ensure_matches!(self.tokens, TokenType::NEG)?;

        Ok(Node::Neg)
    }

    fn parse_eq(&mut self) -> anyhow::Result<Node<'de>> {
        let _ = consume_and_ensure_matches!(self.tokens, TokenType::EQ)?;

        Ok(Node::Eq)
    }

    fn parse_gt(&mut self) -> anyhow::Result<Node<'de>> {
        let _ = consume_and_ensure_matches!(self.tokens, TokenType::GT)?;

        Ok(Node::Gt)
    }

    fn parse_lt(&mut self) -> anyhow::Result<Node<'de>> {
        let _ = consume_and_ensure_matches!(self.tokens, TokenType::LT)?;

        Ok(Node::Lt)
    }

    fn parse_and(&mut self) -> anyhow::Result<Node<'de>> {
        let _ = consume_and_ensure_matches!(self.tokens, TokenType::AND)?;

        Ok(Node::And)
    }

    fn parse_or(&mut self) -> anyhow::Result<Node<'de>> {
        let _ = consume_and_ensure_matches!(self.tokens, TokenType::OR)?;

        Ok(Node::Or)
    }

    fn parse_not(&mut self) -> anyhow::Result<Node<'de>> {
        let _ = consume_and_ensure_matches!(self.tokens, TokenType::NOT)?;

        Ok(Node::Not)
    }

    fn parse_segment(&mut self) -> anyhow::Result<Segment> {
        let token = self.tokens.peek().ok_or(anyhow::anyhow!(
            "Could not consume any more tokens to parse the segment"
        ))?;

        if matches!(token.token_type, TokenType::ARGUMENT) {
            return self.parse_argument_segment();
        }
        if matches!(token.token_type, TokenType::LOCAL) {
            return self.parse_local_segment();
        }
        if matches!(token.token_type, TokenType::STATIC) {
            return self.parse_static_segment();
        }
        if matches!(token.token_type, TokenType::CONSTANT) {
            return self.parse_constant_segment();
        }
        if matches!(token.token_type, TokenType::THIS) {
            return self.parse_this_segment();
        }
        if matches!(token.token_type, TokenType::THAT) {
            return self.parse_that_segment();
        }
        if matches!(token.token_type, TokenType::POINTER) {
            return self.parse_pointer_segment();
        }
        if matches!(token.token_type, TokenType::TEMP) {
            return self.parse_temp_segment();
        }

        anyhow::bail!("Could not parse the segment")
    }

    fn parse_argument_segment(&mut self) -> anyhow::Result<Segment> {
        let _ = consume_and_ensure_matches!(self.tokens, TokenType::ARGUMENT)?;
        let offset = consume_number!(self.tokens)?;

        Ok(Segment::Argument { offset })
    }

    fn parse_local_segment(&mut self) -> anyhow::Result<Segment> {
        let _ = consume_and_ensure_matches!(self.tokens, TokenType::LOCAL)?;
        let offset = consume_number!(self.tokens)?;

        Ok(Segment::Local { offset })
    }

    fn parse_static_segment(&mut self) -> anyhow::Result<Segment> {
        let _ = consume_and_ensure_matches!(self.tokens, TokenType::STATIC)?;
        let offset = consume_number!(self.tokens)?;

        Ok(Segment::Static { offset })
    }

    fn parse_constant_segment(&mut self) -> anyhow::Result<Segment> {
        let _ = consume_and_ensure_matches!(self.tokens, TokenType::CONSTANT)?;
        let value = consume_number!(self.tokens)?;

        Ok(Segment::Constant { value })
    }

    fn parse_this_segment(&mut self) -> anyhow::Result<Segment> {
        let _ = consume_and_ensure_matches!(self.tokens, TokenType::THIS)?;
        let offset = consume_number!(self.tokens)?;

        Ok(Segment::This { offset })
    }

    fn parse_that_segment(&mut self) -> anyhow::Result<Segment> {
        let _ = consume_and_ensure_matches!(self.tokens, TokenType::THAT)?;
        let offset = consume_number!(self.tokens)?;

        Ok(Segment::That { offset })
    }

    fn parse_pointer_segment(&mut self) -> anyhow::Result<Segment> {
        let _ = consume_and_ensure_matches!(self.tokens, TokenType::POINTER)?;
        let offset = consume_number!(self.tokens)?;

        Ok(Segment::Pointer { offset })
    }

    fn parse_temp_segment(&mut self) -> anyhow::Result<Segment> {
        let _ = consume_and_ensure_matches!(self.tokens, TokenType::TEMP)?;
        let offset = consume_number!(self.tokens)?;

        Ok(Segment::Temp { offset })
    }
}

impl<'de, I> Iterator for Parser<'de, I>
where
    I: Iterator<Item = Token<'de>>,
{
    type Item = anyhow::Result<Node<'de>>;

    fn next(&mut self) -> Option<Self::Item> {
        self.parse()
    }
}
