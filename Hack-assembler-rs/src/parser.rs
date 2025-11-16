use std::iter::Peekable;

use crate::scanner::{Token, TokenType};

macro_rules! consume {
    ($tokens:expr) => {
        $tokens.next().ok_or(anyhow::anyhow!(
            "Could not consume a token. Token list is empty"
        ))
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

macro_rules! peek_matches {
    ($tokens:expr, $( $pattern:pat ),* $(,)?) => {
        if let Some(value) = $tokens.peek() {
            match value {
                $(Token {
                    token_type: $pattern,
                    ..
                } => true, )*
                _ => false,
            }
        } else {
            false
        }
    };
}

macro_rules! consume_if_matches {
    ($tokens:expr, $( $pattern:pat ),* $(,)?) => {
        if peek_matches!($tokens, $( $pattern ),*) {
            Some(consume_and_ensure_matches!($tokens, $( $pattern ),*)?)
        } else {
            None
        }
    };
}

pub type Address = u16;

#[derive(Debug)]
pub enum Instruction<'de> {
    /// A-Instruction
    /// Format: @value
    /// Where calue is either a non-negative decimal number
    /// or a symbol referring to such number.
    A {
        _at: Token<'de>,
        /// Either a symbol (label or variable) or a numeric identifier.
        token: Token<'de>,
    },
    /// C-Instruction
    /// Format: dest=comp;jump
    /// Either the dest or jump fields may be empty.
    /// If dest is empty, the "=" is omitted;
    /// If jump is empty, the ";" is omitted.
    C {
        dest: Option<Token<'de>>,
        _eq: Option<Token<'de>>,
        comp: Vec<Token<'de>>,
        _sem: Option<Token<'de>>,
        jump: Option<Token<'de>>,
    },
}

#[derive(Debug)]
pub enum Node<'de> {
    Label {
        _left_paren: Token<'de>,
        name: Token<'de>,
        _right_paren: Token<'de>,
    },
    Instruction(Instruction<'de>),
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

            if matches!(token.token_type, TokenType::LEFT_PAREN) {
                return Some(self.parse_label());
            }

            if matches!(token.token_type, TokenType::AT) {
                return Some(self.parse_a_instruction());
            }

            return Some(self.parse_c_instruction());
        }

        unreachable!()
    }

    fn parse_label(&mut self) -> anyhow::Result<Node<'de>> {
        let _left_paren = consume_and_ensure_matches!(self.tokens, TokenType::LEFT_PAREN)?;
        let name = consume_and_ensure_matches!(self.tokens, TokenType::IDENTIFIER)?;
        let _right_paren = consume_and_ensure_matches!(self.tokens, TokenType::RIGHT_PAREN)?;

        Ok(Node::Label {
            _left_paren,
            name,
            _right_paren,
        })
    }

    fn parse_a_instruction(&mut self) -> anyhow::Result<Node<'de>> {
        let _at = consume_and_ensure_matches!(self.tokens, TokenType::AT)?;
        let token =
            consume_and_ensure_matches!(self.tokens, TokenType::IDENTIFIER | TokenType::NUMBER(_))?;

        Ok(Node::Instruction(Instruction::A { _at, token }))
    }

    fn parse_c_instruction(&mut self) -> anyhow::Result<Node<'de>> {
        fn should_consume_more_for_comp<'de, I: Iterator<Item = Token<'de>>>(
            tokens: &mut Peekable<I>,
            consumed_tokens: &mut Vec<Token<'de>>,
        ) -> anyhow::Result<bool> {
            if let Some(prev) = consumed_tokens.last() {
                // Unary bang
                if matches!(prev.token_type, TokenType::BANG) && consumed_tokens.len() == 1 {
                    let next = consume_and_ensure_matches!(
                        tokens,
                        TokenType::A | TokenType::D | TokenType::M
                    )?;

                    consumed_tokens.push(next);

                    return Ok(false);
                }
                // Unary minus
                if matches!(prev.token_type, TokenType::MINUS) && consumed_tokens.len() == 1 {
                    let next = consume_and_ensure_matches!(
                        tokens,
                        TokenType::A | TokenType::D | TokenType::M | TokenType::NUMBER(1)
                    )?;

                    consumed_tokens.push(next);

                    return Ok(false);
                }
                // Binary. Consume a second operand: MINUS or PLUS
                if matches!(prev.token_type, TokenType::MINUS | TokenType::PLUS)
                    && consumed_tokens.len() == 2
                {
                    let next = consume_and_ensure_matches!(
                        tokens,
                        TokenType::A | TokenType::D | TokenType::M | TokenType::NUMBER(1)
                    )?;

                    consumed_tokens.push(next);

                    return Ok(false);
                }
                // Binary. Consume a second operand: AMPERSAND or BAR
                if matches!(prev.token_type, TokenType::AMPERSAND | TokenType::BAR)
                    && consumed_tokens.len() == 2
                {
                    let next = consume_and_ensure_matches!(tokens, TokenType::A | TokenType::M)?;

                    consumed_tokens.push(next);

                    return Ok(false);
                }
                // Binary. Consume an operator
                if matches!(prev.token_type, TokenType::A | TokenType::D | TokenType::M)
                    && consumed_tokens.len() == 1
                {
                    if let Some(next) = consume_if_matches!(
                        tokens,
                        TokenType::PLUS | TokenType::MINUS | TokenType::AMPERSAND | TokenType::BAR
                    ) {
                        consumed_tokens.push(next);

                        return Ok(true);
                    } else {
                        // Not a binary operation. Nothing to consume more
                        return Ok(false);
                    }
                }

                return Ok(false);
            } else {
                let next = consume!(tokens)?;
                consumed_tokens.push(next);

                return Ok(true);
            }
        }

        let mut dest = None;
        let mut comp = vec![];
        let mut jump = None;

        let mut _eq = None;
        let mut _sem = None;

        enum ParsingState {
            Initial,
            ConsumingComp,
            FinishConsumingComp,
            ConsumingJump,
        }

        let mut state = ParsingState::Initial;
        let mut consumed_tokens = vec![];

        'parsing_loop: loop {
            if peek_matches!(self.tokens, TokenType::EOF) {
                // TODO: verify `comp` is valid
                comp = consumed_tokens;

                break 'parsing_loop;
            }

            if let Some(eq) = consume_if_matches!(self.tokens, TokenType::EQUAL) {
                // TODO: verify `dest` is valid;
                assert!(consumed_tokens.len() == 1);
                dest = consumed_tokens.pop();
                consumed_tokens = vec![];

                state = ParsingState::ConsumingComp;
                _eq = Some(eq);
            } else if let Some(sem) = consume_if_matches!(self.tokens, TokenType::SEMICOLON) {
                // TODO: verify `comp` is valid;
                comp = consumed_tokens;
                consumed_tokens = vec![];

                state = ParsingState::ConsumingJump;
                _sem = Some(sem);
            } else {
                match state {
                    ParsingState::Initial => {
                        consumed_tokens.push(consume!(self.tokens)?);
                    }
                    ParsingState::ConsumingComp => {
                        if !should_consume_more_for_comp(&mut self.tokens, &mut consumed_tokens)? {
                            state = ParsingState::FinishConsumingComp;
                        }
                    }
                    ParsingState::FinishConsumingComp => {
                        comp = consumed_tokens;

                        break 'parsing_loop;
                    }
                    ParsingState::ConsumingJump => {
                        assert!(consumed_tokens.is_empty());
                        // TODO: verify `jump` is valid;
                        jump = Some(consume!(self.tokens)?);

                        break 'parsing_loop;
                    }
                }
            }
        }

        Ok(Node::Instruction(Instruction::C {
            dest,
            _eq,
            comp,
            _sem,
            jump,
        }))
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

#[cfg(test)]
mod single_comp_tests {
    use crate::parser::Parser;

    use super::*;

    fn parse_nodes(tokens: Vec<Token<'_>>) -> Vec<Node<'_>> {
        let parser = Parser::new(tokens.into_iter());
        let nodes: Result<Vec<_>, _> = parser.into_iter().collect();

        nodes.unwrap()
    }

    #[test]
    fn case_0() {
        let tokens = vec![
            Token::new(TokenType::NUMBER(0), "0", 1),
            Token::new(TokenType::EQUAL, "=", 1),
            Token::new(TokenType::NUMBER(0), "0", 1),
            Token::new(TokenType::EOF, "eof", 1),
        ];
        let nodes = parse_nodes(tokens);
        assert_eq!(nodes.len(), 1);

        let Node::Instruction(Instruction::C { comp, .. }) = &nodes[0] else {
            return assert!(false);
        };

        assert_eq!(comp.len(), 1);
        assert!(matches!(
            comp[0],
            Token {
                token_type: TokenType::NUMBER(0),
                ..
            }
        ));
    }

    #[test]
    fn case_1() {
        let tokens = vec![
            Token::new(TokenType::NUMBER(0), "0", 1),
            Token::new(TokenType::EQUAL, "=", 1),
            Token::new(TokenType::NUMBER(1), "1", 1),
            Token::new(TokenType::EOF, "eof", 1),
        ];
        let nodes = parse_nodes(tokens);
        assert_eq!(nodes.len(), 1);

        let Node::Instruction(Instruction::C { comp, .. }) = &nodes[0] else {
            return assert!(false);
        };

        assert_eq!(comp.len(), 1);
        assert!(matches!(
            comp[0],
            Token {
                token_type: TokenType::NUMBER(1),
                ..
            }
        ));
    }

    #[test]
    fn case_minus_1() {
        let tokens = vec![
            Token::new(TokenType::NUMBER(0), "0", 1),
            Token::new(TokenType::EQUAL, "=", 1),
            Token::new(TokenType::MINUS, "-", 1),
            Token::new(TokenType::NUMBER(1), "1", 1),
            Token::new(TokenType::EOF, "eof", 1),
        ];
        let nodes = parse_nodes(tokens);
        assert_eq!(nodes.len(), 1);

        let Node::Instruction(Instruction::C { comp, .. }) = &nodes[0] else {
            return assert!(false);
        };

        assert_eq!(comp.len(), 2);
        assert!(matches!(
            comp[0],
            Token {
                token_type: TokenType::MINUS,
                ..
            }
        ));
        assert!(matches!(
            comp[1],
            Token {
                token_type: TokenType::NUMBER(1),
                ..
            }
        ));
    }

    #[test]
    fn case_d() {
        let tokens = vec![
            Token::new(TokenType::NUMBER(0), "0", 1),
            Token::new(TokenType::EQUAL, "=", 1),
            Token::new(TokenType::D, "D", 1),
            Token::new(TokenType::EOF, "eof", 1),
        ];
        let nodes = parse_nodes(tokens);
        assert_eq!(nodes.len(), 1);

        let Node::Instruction(Instruction::C { comp, .. }) = &nodes[0] else {
            return assert!(false);
        };

        assert_eq!(comp.len(), 1);
        assert!(matches!(
            comp[0],
            Token {
                token_type: TokenType::D,
                ..
            }
        ));
    }

    #[test]
    fn case_a() {
        let tokens = vec![
            Token::new(TokenType::NUMBER(0), "0", 1),
            Token::new(TokenType::EQUAL, "=", 1),
            Token::new(TokenType::A, "A", 1),
            Token::new(TokenType::EOF, "eof", 1),
        ];
        let nodes = parse_nodes(tokens);
        assert_eq!(nodes.len(), 1);

        let Node::Instruction(Instruction::C { comp, .. }) = &nodes[0] else {
            return assert!(false);
        };

        assert_eq!(comp.len(), 1);
        assert!(matches!(
            comp[0],
            Token {
                token_type: TokenType::A,
                ..
            }
        ));
    }

    #[test]
    fn case_not_d() {
        let tokens = vec![
            Token::new(TokenType::NUMBER(0), "0", 1),
            Token::new(TokenType::EQUAL, "=", 1),
            Token::new(TokenType::BANG, "!", 1),
            Token::new(TokenType::D, "D", 1),
            Token::new(TokenType::EOF, "eof", 1),
        ];
        let nodes = parse_nodes(tokens);
        assert_eq!(nodes.len(), 1);

        let Node::Instruction(Instruction::C { comp, .. }) = &nodes[0] else {
            return assert!(false);
        };

        assert_eq!(comp.len(), 2);
        assert!(matches!(
            comp[0],
            Token {
                token_type: TokenType::BANG,
                ..
            }
        ));
        assert!(matches!(
            comp[1],
            Token {
                token_type: TokenType::D,
                ..
            }
        ));
    }

    #[test]
    fn case_not_a() {
        let tokens = vec![
            Token::new(TokenType::NUMBER(0), "0", 1),
            Token::new(TokenType::EQUAL, "=", 1),
            Token::new(TokenType::BANG, "!", 1),
            Token::new(TokenType::A, "A", 1),
            Token::new(TokenType::EOF, "eof", 1),
        ];
        let nodes = parse_nodes(tokens);
        assert_eq!(nodes.len(), 1);

        let Node::Instruction(Instruction::C { comp, .. }) = &nodes[0] else {
            return assert!(false);
        };

        assert_eq!(comp.len(), 2);
        assert!(matches!(
            comp[0],
            Token {
                token_type: TokenType::BANG,
                ..
            }
        ));
        assert!(matches!(
            comp[1],
            Token {
                token_type: TokenType::A,
                ..
            }
        ));
    }

    #[test]
    fn case_minus_d() {
        let tokens = vec![
            Token::new(TokenType::NUMBER(0), "0", 1),
            Token::new(TokenType::EQUAL, "=", 1),
            Token::new(TokenType::MINUS, "-", 1),
            Token::new(TokenType::D, "D", 1),
            Token::new(TokenType::EOF, "eof", 1),
        ];
        let nodes = parse_nodes(tokens);
        assert_eq!(nodes.len(), 1);

        let Node::Instruction(Instruction::C { comp, .. }) = &nodes[0] else {
            return assert!(false);
        };

        assert_eq!(comp.len(), 2);
        assert!(matches!(
            comp[0],
            Token {
                token_type: TokenType::MINUS,
                ..
            }
        ));
        assert!(matches!(
            comp[1],
            Token {
                token_type: TokenType::D,
                ..
            }
        ));
    }

    #[test]
    fn case_minus_a() {
        let tokens = vec![
            Token::new(TokenType::NUMBER(0), "0", 1),
            Token::new(TokenType::EQUAL, "=", 1),
            Token::new(TokenType::MINUS, "-", 1),
            Token::new(TokenType::A, "A", 1),
            Token::new(TokenType::EOF, "eof", 1),
        ];
        let nodes = parse_nodes(tokens);
        assert_eq!(nodes.len(), 1);

        let Node::Instruction(Instruction::C { comp, .. }) = &nodes[0] else {
            return assert!(false);
        };

        assert_eq!(comp.len(), 2);
        assert!(matches!(
            comp[0],
            Token {
                token_type: TokenType::MINUS,
                ..
            }
        ));
        assert!(matches!(
            comp[1],
            Token {
                token_type: TokenType::A,
                ..
            }
        ));
    }

    #[test]
    fn case_d_plus_1() {
        let tokens = vec![
            Token::new(TokenType::NUMBER(0), "0", 1),
            Token::new(TokenType::EQUAL, "=", 1),
            Token::new(TokenType::D, "D", 1),
            Token::new(TokenType::PLUS, "+", 1),
            Token::new(TokenType::NUMBER(1), "1", 1),
            Token::new(TokenType::EOF, "eof", 1),
        ];
        let nodes = parse_nodes(tokens);
        assert_eq!(nodes.len(), 1);

        let Node::Instruction(Instruction::C { comp, .. }) = &nodes[0] else {
            return assert!(false);
        };

        assert_eq!(comp.len(), 3);
        assert!(matches!(
            comp[0],
            Token {
                token_type: TokenType::D,
                ..
            }
        ));
        assert!(matches!(
            comp[1],
            Token {
                token_type: TokenType::PLUS,
                ..
            }
        ));
        assert!(matches!(
            comp[2],
            Token {
                token_type: TokenType::NUMBER(1),
                ..
            }
        ));
    }

    #[test]
    fn case_a_plus_1() {
        let tokens = vec![
            Token::new(TokenType::NUMBER(0), "0", 1),
            Token::new(TokenType::EQUAL, "=", 1),
            Token::new(TokenType::A, "A", 1),
            Token::new(TokenType::PLUS, "+", 1),
            Token::new(TokenType::NUMBER(1), "1", 1),
            Token::new(TokenType::EOF, "eof", 1),
        ];
        let nodes = parse_nodes(tokens);
        assert_eq!(nodes.len(), 1);

        let Node::Instruction(Instruction::C { comp, .. }) = &nodes[0] else {
            return assert!(false);
        };

        assert_eq!(comp.len(), 3);
        assert!(matches!(
            comp[0],
            Token {
                token_type: TokenType::A,
                ..
            }
        ));
        assert!(matches!(
            comp[1],
            Token {
                token_type: TokenType::PLUS,
                ..
            }
        ));
        assert!(matches!(
            comp[2],
            Token {
                token_type: TokenType::NUMBER(1),
                ..
            }
        ));
    }

    #[test]
    fn case_d_minus_1() {
        let tokens = vec![
            Token::new(TokenType::NUMBER(0), "0", 1),
            Token::new(TokenType::EQUAL, "=", 1),
            Token::new(TokenType::D, "D", 1),
            Token::new(TokenType::MINUS, "-", 1),
            Token::new(TokenType::NUMBER(1), "1", 1),
            Token::new(TokenType::EOF, "eof", 1),
        ];
        let nodes = parse_nodes(tokens);
        assert_eq!(nodes.len(), 1);

        let Node::Instruction(Instruction::C { comp, .. }) = &nodes[0] else {
            return assert!(false);
        };

        assert_eq!(comp.len(), 3);
        assert!(matches!(
            comp[0],
            Token {
                token_type: TokenType::D,
                ..
            }
        ));
        assert!(matches!(
            comp[1],
            Token {
                token_type: TokenType::MINUS,
                ..
            }
        ));
        assert!(matches!(
            comp[2],
            Token {
                token_type: TokenType::NUMBER(1),
                ..
            }
        ));
    }

    #[test]
    fn case_a_minus_1() {
        let tokens = vec![
            Token::new(TokenType::NUMBER(0), "0", 1),
            Token::new(TokenType::EQUAL, "=", 1),
            Token::new(TokenType::A, "A", 1),
            Token::new(TokenType::MINUS, "-", 1),
            Token::new(TokenType::NUMBER(1), "1", 1),
            Token::new(TokenType::EOF, "eof", 1),
        ];
        let nodes = parse_nodes(tokens);
        assert_eq!(nodes.len(), 1);

        let Node::Instruction(Instruction::C { comp, .. }) = &nodes[0] else {
            return assert!(false);
        };

        assert_eq!(comp.len(), 3);
        assert!(matches!(
            comp[0],
            Token {
                token_type: TokenType::A,
                ..
            }
        ));
        assert!(matches!(
            comp[1],
            Token {
                token_type: TokenType::MINUS,
                ..
            }
        ));
        assert!(matches!(
            comp[2],
            Token {
                token_type: TokenType::NUMBER(1),
                ..
            }
        ));
    }

    #[test]
    fn case_d_plus_a() {
        let tokens = vec![
            Token::new(TokenType::NUMBER(0), "0", 1),
            Token::new(TokenType::EQUAL, "=", 1),
            Token::new(TokenType::D, "D", 1),
            Token::new(TokenType::PLUS, "+", 1),
            Token::new(TokenType::A, "A", 1),
            Token::new(TokenType::EOF, "eof", 1),
        ];
        let nodes = parse_nodes(tokens);
        assert_eq!(nodes.len(), 1);

        let Node::Instruction(Instruction::C { comp, .. }) = &nodes[0] else {
            return assert!(false);
        };

        assert_eq!(comp.len(), 3);
        assert!(matches!(
            comp[0],
            Token {
                token_type: TokenType::D,
                ..
            }
        ));
        assert!(matches!(
            comp[1],
            Token {
                token_type: TokenType::PLUS,
                ..
            }
        ));
        assert!(matches!(
            comp[2],
            Token {
                token_type: TokenType::A,
                ..
            }
        ));
    }

    #[test]
    fn case_d_minus_a() {
        let tokens = vec![
            Token::new(TokenType::NUMBER(0), "0", 1),
            Token::new(TokenType::EQUAL, "=", 1),
            Token::new(TokenType::D, "D", 1),
            Token::new(TokenType::MINUS, "-", 1),
            Token::new(TokenType::A, "A", 1),
            Token::new(TokenType::EOF, "eof", 1),
        ];
        let nodes = parse_nodes(tokens);
        assert_eq!(nodes.len(), 1);

        let Node::Instruction(Instruction::C { comp, .. }) = &nodes[0] else {
            return assert!(false);
        };

        assert_eq!(comp.len(), 3);
        assert!(matches!(
            comp[0],
            Token {
                token_type: TokenType::D,
                ..
            }
        ));
        assert!(matches!(
            comp[1],
            Token {
                token_type: TokenType::MINUS,
                ..
            }
        ));
        assert!(matches!(
            comp[2],
            Token {
                token_type: TokenType::A,
                ..
            }
        ));
    }

    #[test]
    fn case_a_minus_d() {
        let tokens = vec![
            Token::new(TokenType::NUMBER(0), "0", 1),
            Token::new(TokenType::EQUAL, "=", 1),
            Token::new(TokenType::A, "A", 1),
            Token::new(TokenType::MINUS, "-", 1),
            Token::new(TokenType::D, "D", 1),
            Token::new(TokenType::EOF, "eof", 1),
        ];
        let nodes = parse_nodes(tokens);
        assert_eq!(nodes.len(), 1);

        let Node::Instruction(Instruction::C { comp, .. }) = &nodes[0] else {
            return assert!(false);
        };

        assert_eq!(comp.len(), 3);
        assert!(matches!(
            comp[0],
            Token {
                token_type: TokenType::A,
                ..
            }
        ));
        assert!(matches!(
            comp[1],
            Token {
                token_type: TokenType::MINUS,
                ..
            }
        ));
        assert!(matches!(
            comp[2],
            Token {
                token_type: TokenType::D,
                ..
            }
        ));
    }

    #[test]
    fn case_d_and_a() {
        let tokens = vec![
            Token::new(TokenType::NUMBER(0), "0", 1),
            Token::new(TokenType::EQUAL, "=", 1),
            Token::new(TokenType::D, "D", 1),
            Token::new(TokenType::AMPERSAND, "&", 1),
            Token::new(TokenType::A, "D", 1),
            Token::new(TokenType::EOF, "eof", 1),
        ];
        let nodes = parse_nodes(tokens);
        assert_eq!(nodes.len(), 1);

        let Node::Instruction(Instruction::C { comp, .. }) = &nodes[0] else {
            return assert!(false);
        };

        assert_eq!(comp.len(), 3);
        assert!(matches!(
            comp[0],
            Token {
                token_type: TokenType::D,
                ..
            }
        ));
        assert!(matches!(
            comp[1],
            Token {
                token_type: TokenType::AMPERSAND,
                ..
            }
        ));
        assert!(matches!(
            comp[2],
            Token {
                token_type: TokenType::A,
                ..
            }
        ));
    }

    #[test]
    fn case_d_or_a() {
        let tokens = vec![
            Token::new(TokenType::NUMBER(0), "0", 1),
            Token::new(TokenType::EQUAL, "=", 1),
            Token::new(TokenType::D, "D", 1),
            Token::new(TokenType::BAR, "|", 1),
            Token::new(TokenType::A, "D", 1),
            Token::new(TokenType::EOF, "eof", 1),
        ];
        let nodes = parse_nodes(tokens);
        assert_eq!(nodes.len(), 1);

        let Node::Instruction(Instruction::C { comp, .. }) = &nodes[0] else {
            return assert!(false);
        };

        assert_eq!(comp.len(), 3);
        assert!(matches!(
            comp[0],
            Token {
                token_type: TokenType::D,
                ..
            }
        ));
        assert!(matches!(
            comp[1],
            Token {
                token_type: TokenType::BAR,
                ..
            }
        ));
        assert!(matches!(
            comp[2],
            Token {
                token_type: TokenType::A,
                ..
            }
        ));
    }

    #[test]
    fn case_m() {
        let tokens = vec![
            Token::new(TokenType::NUMBER(0), "0", 1),
            Token::new(TokenType::EQUAL, "=", 1),
            Token::new(TokenType::M, "M", 1),
            Token::new(TokenType::EOF, "eof", 1),
        ];
        let nodes = parse_nodes(tokens);
        assert_eq!(nodes.len(), 1);

        let Node::Instruction(Instruction::C { comp, .. }) = &nodes[0] else {
            return assert!(false);
        };

        assert_eq!(comp.len(), 1);
        assert!(matches!(
            comp[0],
            Token {
                token_type: TokenType::M,
                ..
            }
        ));
    }

    #[test]
    fn case_not_m() {
        let tokens = vec![
            Token::new(TokenType::NUMBER(0), "0", 1),
            Token::new(TokenType::EQUAL, "=", 1),
            Token::new(TokenType::BANG, "!", 1),
            Token::new(TokenType::M, "M", 1),
            Token::new(TokenType::EOF, "eof", 1),
        ];
        let nodes = parse_nodes(tokens);
        assert_eq!(nodes.len(), 1);

        let Node::Instruction(Instruction::C { comp, .. }) = &nodes[0] else {
            return assert!(false);
        };

        assert_eq!(comp.len(), 2);
        assert!(matches!(
            comp[0],
            Token {
                token_type: TokenType::BANG,
                ..
            }
        ));
        assert!(matches!(
            comp[1],
            Token {
                token_type: TokenType::M,
                ..
            }
        ));
    }

    #[test]
    fn case_minus_m() {
        let tokens = vec![
            Token::new(TokenType::NUMBER(0), "0", 1),
            Token::new(TokenType::EQUAL, "=", 1),
            Token::new(TokenType::MINUS, "-", 1),
            Token::new(TokenType::M, "M", 1),
            Token::new(TokenType::EOF, "eof", 1),
        ];
        let nodes = parse_nodes(tokens);
        assert_eq!(nodes.len(), 1);

        let Node::Instruction(Instruction::C { comp, .. }) = &nodes[0] else {
            return assert!(false);
        };

        assert_eq!(comp.len(), 2);
        assert!(matches!(
            comp[0],
            Token {
                token_type: TokenType::MINUS,
                ..
            }
        ));
        assert!(matches!(
            comp[1],
            Token {
                token_type: TokenType::M,
                ..
            }
        ));
    }

    #[test]
    fn case_m_plus_1() {
        let tokens = vec![
            Token::new(TokenType::NUMBER(0), "0", 1),
            Token::new(TokenType::EQUAL, "=", 1),
            Token::new(TokenType::M, "M", 1),
            Token::new(TokenType::PLUS, "+", 1),
            Token::new(TokenType::NUMBER(1), "1", 1),
            Token::new(TokenType::EOF, "eof", 1),
        ];
        let nodes = parse_nodes(tokens);
        assert_eq!(nodes.len(), 1);

        let Node::Instruction(Instruction::C { comp, .. }) = &nodes[0] else {
            return assert!(false);
        };

        assert_eq!(comp.len(), 3);
        assert!(matches!(
            comp[0],
            Token {
                token_type: TokenType::M,
                ..
            }
        ));
        assert!(matches!(
            comp[1],
            Token {
                token_type: TokenType::PLUS,
                ..
            }
        ));
        assert!(matches!(
            comp[2],
            Token {
                token_type: TokenType::NUMBER(1),
                ..
            }
        ));
    }

    #[test]
    fn case_m_minus_1() {
        let tokens = vec![
            Token::new(TokenType::NUMBER(0), "0", 1),
            Token::new(TokenType::EQUAL, "=", 1),
            Token::new(TokenType::M, "M", 1),
            Token::new(TokenType::MINUS, "-", 1),
            Token::new(TokenType::NUMBER(1), "1", 1),
            Token::new(TokenType::EOF, "eof", 1),
        ];
        let nodes = parse_nodes(tokens);
        assert_eq!(nodes.len(), 1);

        let Node::Instruction(Instruction::C { comp, .. }) = &nodes[0] else {
            return assert!(false);
        };

        assert_eq!(comp.len(), 3);
        assert!(matches!(
            comp[0],
            Token {
                token_type: TokenType::M,
                ..
            }
        ));
        assert!(matches!(
            comp[1],
            Token {
                token_type: TokenType::MINUS,
                ..
            }
        ));
        assert!(matches!(
            comp[2],
            Token {
                token_type: TokenType::NUMBER(1),
                ..
            }
        ));
    }

    #[test]
    fn case_d_plus_m() {
        let tokens = vec![
            Token::new(TokenType::NUMBER(0), "0", 1),
            Token::new(TokenType::EQUAL, "=", 1),
            Token::new(TokenType::D, "D", 1),
            Token::new(TokenType::PLUS, "+", 1),
            Token::new(TokenType::M, "M", 1),
            Token::new(TokenType::EOF, "eof", 1),
        ];
        let nodes = parse_nodes(tokens);
        assert_eq!(nodes.len(), 1);

        let Node::Instruction(Instruction::C { comp, .. }) = &nodes[0] else {
            return assert!(false);
        };

        assert_eq!(comp.len(), 3);
        assert!(matches!(
            comp[0],
            Token {
                token_type: TokenType::D,
                ..
            }
        ));
        assert!(matches!(
            comp[1],
            Token {
                token_type: TokenType::PLUS,
                ..
            }
        ));
        assert!(matches!(
            comp[2],
            Token {
                token_type: TokenType::M,
                ..
            }
        ));
    }

    #[test]
    fn case_d_minus_m() {
        let tokens = vec![
            Token::new(TokenType::NUMBER(0), "0", 1),
            Token::new(TokenType::EQUAL, "=", 1),
            Token::new(TokenType::D, "D", 1),
            Token::new(TokenType::MINUS, "-", 1),
            Token::new(TokenType::M, "M", 1),
            Token::new(TokenType::EOF, "eof", 1),
        ];
        let nodes = parse_nodes(tokens);
        assert_eq!(nodes.len(), 1);

        let Node::Instruction(Instruction::C { comp, .. }) = &nodes[0] else {
            return assert!(false);
        };

        assert_eq!(comp.len(), 3);
        assert!(matches!(
            comp[0],
            Token {
                token_type: TokenType::D,
                ..
            }
        ));
        assert!(matches!(
            comp[1],
            Token {
                token_type: TokenType::MINUS,
                ..
            }
        ));
        assert!(matches!(
            comp[2],
            Token {
                token_type: TokenType::M,
                ..
            }
        ));
    }

    #[test]
    fn case_m_minus_d() {
        let tokens = vec![
            Token::new(TokenType::NUMBER(0), "0", 1),
            Token::new(TokenType::EQUAL, "=", 1),
            Token::new(TokenType::M, "M", 1),
            Token::new(TokenType::MINUS, "-", 1),
            Token::new(TokenType::D, "D", 1),
            Token::new(TokenType::EOF, "eof", 1),
        ];
        let nodes = parse_nodes(tokens);
        assert_eq!(nodes.len(), 1);

        let Node::Instruction(Instruction::C { comp, .. }) = &nodes[0] else {
            return assert!(false);
        };

        assert_eq!(comp.len(), 3);
        assert!(matches!(
            comp[0],
            Token {
                token_type: TokenType::M,
                ..
            }
        ));
        assert!(matches!(
            comp[1],
            Token {
                token_type: TokenType::MINUS,
                ..
            }
        ));
        assert!(matches!(
            comp[2],
            Token {
                token_type: TokenType::D,
                ..
            }
        ));
    }

    #[test]
    fn case_d_and_m() {
        let tokens = vec![
            Token::new(TokenType::NUMBER(0), "0", 1),
            Token::new(TokenType::EQUAL, "=", 1),
            Token::new(TokenType::D, "D", 1),
            Token::new(TokenType::AMPERSAND, "&", 1),
            Token::new(TokenType::M, "M", 1),
            Token::new(TokenType::EOF, "eof", 1),
        ];
        let nodes = parse_nodes(tokens);
        assert_eq!(nodes.len(), 1);

        let Node::Instruction(Instruction::C { comp, .. }) = &nodes[0] else {
            return assert!(false);
        };

        assert_eq!(comp.len(), 3);
        assert!(matches!(
            comp[0],
            Token {
                token_type: TokenType::D,
                ..
            }
        ));
        assert!(matches!(
            comp[1],
            Token {
                token_type: TokenType::AMPERSAND,
                ..
            }
        ));
        assert!(matches!(
            comp[2],
            Token {
                token_type: TokenType::M,
                ..
            }
        ));
    }

    #[test]
    fn case_d_or_m() {
        let tokens = vec![
            Token::new(TokenType::NUMBER(0), "0", 1),
            Token::new(TokenType::EQUAL, "=", 1),
            Token::new(TokenType::D, "D", 1),
            Token::new(TokenType::BAR, "|", 1),
            Token::new(TokenType::M, "M", 1),
            Token::new(TokenType::EOF, "eof", 1),
        ];
        let nodes = parse_nodes(tokens);
        assert_eq!(nodes.len(), 1);

        let Node::Instruction(Instruction::C { comp, .. }) = &nodes[0] else {
            return assert!(false);
        };

        assert_eq!(comp.len(), 3);
        assert!(matches!(
            comp[0],
            Token {
                token_type: TokenType::D,
                ..
            }
        ));
        assert!(matches!(
            comp[1],
            Token {
                token_type: TokenType::BAR,
                ..
            }
        ));
        assert!(matches!(
            comp[2],
            Token {
                token_type: TokenType::M,
                ..
            }
        ));
    }
}

#[cfg(test)]
mod a_tests {
    use crate::parser::Parser;

    use super::*;

    fn parse_nodes(tokens: Vec<Token<'_>>) -> Vec<Node<'_>> {
        let parser = Parser::new(tokens.into_iter());
        let nodes: Result<Vec<_>, _> = parser.into_iter().collect();

        nodes.unwrap()
    }

    #[test]
    fn init() {
        let tokens = vec![
            Token::new(TokenType::AT, "@", 1),
            Token::new(TokenType::NUMBER(10), "10", 1),
            Token::new(TokenType::EOF, "eof", 1),
        ];
        let nodes = parse_nodes(tokens);

        assert_eq!(nodes.len(), 1);
        assert!(matches!(
            nodes[0],
            Node::Instruction(Instruction::A {
                _at: Token {
                    token_type: TokenType::AT,
                    ..
                },
                token: Token {
                    token_type: TokenType::NUMBER(10),
                    ..
                },
            }),
        ));
    }
}
