/// A-instruction:
/// @value
/// Binary: 0vvv|vvvv|vvvv|vvvv

/// C-instruction:
/// dest=comp;jump
/// Binary: 111a|c1c2c3c4|c5c6d1d2|d3j1j2j3
/*
** comp **
| comp | a | c1 | c2 | c3 | c4 | c5 | c6 | Decimal |
| ---- | - | -- | -- | -- | -- | -- | -- | ------- |
| 0    | 0 | 1  | 0  | 1  | 0  | 1  | 0  | 42      |
| 1    | 0 | 1  | 1  | 1  | 1  | 1  | 1  | 63      |
| -1   | 0 | 1  | 1  | 1  | 0  | 1  | 0  | 58      |
| D    | 0 | 0  | 0  | 1  | 1  | 0  | 0  | 12      |
| A    | 0 | 1  | 1  | 0  | 0  | 0  | 0  | 48      |
| !D   | 0 | 0  | 0  | 1  | 1  | 0  | 1  | 13      |
| !A   | 0 | 1  | 1  | 0  | 0  | 0  | 1  | 49      |
| -D   | 0 | 0  | 0  | 1  | 1  | 1  | 1  | 15      |
| -A   | 0 | 1  | 1  | 0  | 0  | 1  | 1  | 51      |
| D+1  | 0 | 0  | 1  | 1  | 1  | 1  | 1  | 31      |
| A+1  | 0 | 1  | 1  | 0  | 1  | 1  | 1  | 55      |
| D-1  | 0 | 0  | 0  | 1  | 1  | 1  | 0  | 14      |
| A-1  | 0 | 1  | 1  | 0  | 0  | 1  | 0  | 50      |
| D+A  | 0 | 0  | 0  | 0  | 0  | 1  | 0  | 2       |
| D-A  | 0 | 0  | 1  | 0  | 0  | 1  | 1  | 19      |
| A-D  | 0 | 0  | 0  | 0  | 1  | 1  | 1  | 7       |
| D&A  | 0 | 0  | 0  | 0  | 0  | 0  | 0  | 0       |
| D|A  | 0 | 0  | 1  | 0  | 1  | 0  | 1  | 21      |
| M    | 1 | 1  | 1  | 0  | 0  | 0  | 0  | 112     |
| !M   | 1 | 1  | 1  | 0  | 0  | 0  | 1  | 113     |
| -M   | 1 | 1  | 1  | 0  | 0  | 1  | 1  | 115     |
| M+1  | 1 | 1  | 1  | 0  | 1  | 1  | 1  | 119     |
| M-1  | 1 | 1  | 1  | 0  | 0  | 1  | 0  | 114     |
| D+M  | 1 | 0  | 0  | 0  | 0  | 1  | 0  | 66      |
| D-M  | 1 | 0  | 1  | 0  | 0  | 1  | 1  | 83      |
| M-D  | 1 | 0  | 0  | 0  | 1  | 1  | 1  | 71      |
| D&M  | 1 | 0  | 0  | 0  | 0  | 0  | 0  | 64      |
| D|M  | 1 | 0  | 1  | 0  | 1  | 0  | 1  | 85      |

** dest **
| dest | d1 | d2 | d3 | Decimal |
| ---- | -- | -- | -- | ------- |
| null | 0  | 0  | 0  | 0       |
| M    | 0  | 0  | 1  | 1       |
| D    | 0  | 1  | 0  | 2       |
| MD   | 0  | 1  | 1  | 3       |
| A    | 1  | 0  | 0  | 4       |
| AM   | 1  | 0  | 1  | 5       |
| AD   | 1  | 1  | 0  | 6       |
| AMD  | 1  | 1  | 1  | 7       |

** jump **
| jump | j1 | j2 | j3 | Decimal |
| ---- | -- | -- | -- | ------- |
| null | 0  | 0  | 0  | 0       |
| JGT  | 0  | 0  | 1  | 1       |
| JEQ  | 0  | 1  | 0  | 2       |
| JGE  | 0  | 1  | 1  | 3       |
| JLT  | 1  | 0  | 0  | 4       |
| JNE  | 1  | 0  | 1  | 5       |
| JLE  | 1  | 1  | 0  | 6       |
| JMP  | 1  | 1  | 1  | 7       |
*/
use crate::{
    parser::{Address, Instruction, Node},
    scanner::{Token, TokenType},
};

#[derive(Debug)]
pub(crate) struct Assembler<'de, I: IntoIterator<Item = Node<'de>>> {
    nodes: I,
}

impl<'de, I> Assembler<'de, I>
where
    I: IntoIterator<Item = Node<'de>>,
{
    pub fn new(nodes: I) -> Self {
        Self { nodes }
    }

    pub fn assemble(self) -> Vec<Address> {
        let nodes = self.nodes;

        nodes
            .into_iter()
            .map(|node| match node {
                Node::Instruction(instruction) => Assembler::<I>::assemble_instruction(instruction),
                Node::Label { .. } => unreachable!(),
            })
            .collect::<Vec<_>>()
    }

    fn assemble_instruction(instruction: Instruction) -> Address {
        match instruction {
            Instruction::A { token, .. } => match token {
                Token {
                    token_type: TokenType::NUMBER(value),
                    ..
                } => {
                    assert!((value >> 15) == 0);
                    value
                }
                _ => unreachable!(),
            },
            Instruction::C {
                dest, comp, jump, ..
            } => {
                let mut result = 0;

                result |= 1 << 15;
                result |= 1 << 14;
                result |= 1 << 13;

                if let Some(token) = jump {
                    let jump = Assembler::<I>::assemble_jump(&token);
                    result |= jump;
                }

                if let Some(token) = dest {
                    let dest = Assembler::<I>::assemble_dest(&token);
                    result |= dest << 3;
                }

                let comp = Assembler::<I>::assemble_comp(&comp);
                result |= comp << 6;

                result
            }
        }
    }

    #[rustfmt::skip]
    fn assemble_jump(token: &Token<'_>) -> u16 {
        match token {
            Token {
                token_type: TokenType::NUMBER(0), ..
            } => 0,
            Token {
                token_type: TokenType::JGT, ..
            } => 1,
            Token {
                token_type: TokenType::JEQ, ..
            } => 2,
            Token {
                token_type: TokenType::JGE, ..
            } => 3,
            Token {
                token_type: TokenType::JLT, ..
            } => 4,
            Token {
                token_type: TokenType::JNE, ..
            } => 5,
            Token {
                token_type: TokenType::JLE, ..
            } => 6,
            Token {
                token_type: TokenType::JMP, ..
            } => 7,
            _ => unreachable!("Expect a correct `jump` in the assemble step"),
        }
    }

    #[rustfmt::skip]
    fn assemble_dest(token: &Token<'_>) -> u16 {
        match token {
            &Token {
                token_type: TokenType::NUMBER(0), ..
            } => 0,
            &Token {
                token_type: TokenType::M, ..
            } => 1,
            &Token {
                token_type: TokenType::D, ..
            } => 2,
            &Token {
                token_type: TokenType::MD, ..
            } => 3,
            &Token {
                token_type: TokenType::A, ..
            } => 4,
            &Token {
                token_type: TokenType::AM, ..
            } => 5,
            &Token {
                token_type: TokenType::AD, ..
            } => 6,
            &Token {
                token_type: TokenType::AMD, ..
            } => 7,
            _ => unreachable!("Expect a correct `dest` in the assemble step"),
        }
    }

    #[rustfmt::skip]
    fn assemble_comp(tokens: &[Token<'_>]) -> u16 {
        match tokens {
            &[Token {
                token_type: TokenType::NUMBER(0), ..
            }] => 42,
            &[Token {
                token_type: TokenType::NUMBER(1), ..
            }] => 63,
            &[Token {
                token_type: TokenType::MINUS, ..
            },Token {
                token_type: TokenType::NUMBER(1), ..
            }] => 58,
            &[Token {
                token_type: TokenType::D, ..
            }] => 12,
            &[Token {
                token_type: TokenType::A, ..
            }] => 48,
            &[Token {
                token_type: TokenType::BANG, ..
            },Token {
                token_type: TokenType::D, ..
            }] => 13,
            &[Token {
                token_type: TokenType::BANG, ..
            },Token {
                token_type: TokenType::A, ..
            }] => 49,
            &[Token {
                token_type: TokenType::MINUS, ..
            },Token {
                token_type: TokenType::D, ..
            }] => 15,
            &[Token {
                token_type: TokenType::MINUS, ..
            },Token {
                token_type: TokenType::A, ..
            }] => 51,
            &[Token {
                token_type: TokenType::D, ..
            },Token {
                token_type: TokenType::PLUS, ..
            },Token {
                token_type: TokenType::NUMBER(1), ..
            }] => 31,
            &[Token {
                token_type: TokenType::A, ..
            },Token {
                token_type: TokenType::PLUS, ..
            },Token {
                token_type: TokenType::NUMBER(1), ..
            }] => 55,
            &[Token {
                token_type: TokenType::D, ..
            },Token {
                token_type: TokenType::MINUS, ..
            },Token {
                token_type: TokenType::NUMBER(1), ..
            }] => 14,
            &[Token {
                token_type: TokenType::A, ..
            },Token {
                token_type: TokenType::MINUS, ..
            },Token {
                token_type: TokenType::NUMBER(1), ..
            }] => 50,
            &[Token {
                token_type: TokenType::D, ..
            },Token {
                token_type: TokenType::PLUS, ..
            },Token {
                token_type: TokenType::A, ..
            }] => 2,
            &[Token {
                token_type: TokenType::D, ..
            },Token {
                token_type: TokenType::MINUS, ..
            },Token {
                token_type: TokenType::A, ..
            }] => 19,
            &[Token {
                token_type: TokenType::A, ..
            },Token {
                token_type: TokenType::MINUS, ..
            },Token {
                token_type: TokenType::D, ..
            }] => 7,
            &[Token {
                token_type: TokenType::D, ..
            },Token {
                token_type: TokenType::AMPERSAND, ..
            },Token {
                token_type: TokenType::A, ..
            }] => 0,
            &[Token {
                token_type: TokenType::D, ..
            },Token {
                token_type: TokenType::BAR, ..
            },Token {
                token_type: TokenType::A, ..
            }] => 21,
            &[Token {
                token_type: TokenType::M, ..
            }] => 112,
            &[Token {
                token_type: TokenType::BANG, ..
            },Token {
                token_type: TokenType::M, ..
            }] => 113,
            &[Token {
                token_type: TokenType::MINUS, ..
            },Token {
                token_type: TokenType::M, ..
            }] => 115,
            &[Token {
                token_type: TokenType::M, ..
            },Token {
                token_type: TokenType::PLUS, ..
            },Token {
                token_type: TokenType::NUMBER(1), ..
            }] => 119,
            &[Token {
                token_type: TokenType::M, ..
            },Token {
                token_type: TokenType::MINUS, ..
            },Token {
                token_type: TokenType::NUMBER(1), ..
            }] => 114,
            &[Token {
                token_type: TokenType::D, ..
            },Token {
                token_type: TokenType::PLUS, ..
            },Token {
                token_type: TokenType::M, ..
            }] => 66,
            &[Token {
                token_type: TokenType::D, ..
            },Token {
                token_type: TokenType::MINUS, ..
            },Token {
                token_type: TokenType::M, ..
            }] => 83,
            &[Token {
                token_type: TokenType::M, ..
            },Token {
                token_type: TokenType::MINUS, ..
            },Token {
                token_type: TokenType::D, ..
            }] => 71,
            &[Token {
                token_type: TokenType::D, ..
            },Token {
                token_type: TokenType::AMPERSAND, ..
            },Token {
                token_type: TokenType::M, ..
            }] => 64,
            &[Token {
                token_type: TokenType::D, ..
            },Token {
                token_type: TokenType::BAR, ..
            },Token {
                token_type: TokenType::M, ..
            }] => 85,
            _ => unreachable!("Expect a correct `comp` in the assemble step"),
        }
    }
}
