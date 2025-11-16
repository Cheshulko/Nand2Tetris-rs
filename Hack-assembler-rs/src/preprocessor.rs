use std::{borrow::Cow, collections::HashMap};

use crate::{
    parser::{Address, Node},
    scanner::{Token, TokenType},
};

#[allow(unused)]
#[derive(Debug)]
pub(crate) struct InitialState;

#[derive(Debug)]
pub(crate) struct StaticSymbolInited;

#[derive(Debug)]
pub(crate) struct SymbolExtractedState;

#[allow(unused)]
#[derive(Debug)]
pub(crate) struct SymbolReplacedState;

type SymbolTable<'a> = HashMap<Cow<'a, str>, Address>;

#[derive(Debug)]
pub(crate) struct Preprocessor<'de, I, State> {
    nodes: I,
    symbol_table: SymbolTable<'de>,
    next_free_memory_address: Address,
    _marker: std::marker::PhantomData<State>,
}

impl<'de, I, S> Preprocessor<'de, I, S> {
    pub fn symbol_table(&self) -> &SymbolTable<'de> {
        return &self.symbol_table;
    }
}

impl<'de, I> Preprocessor<'de, I, InitialState>
where
    I: IntoIterator<Item = Node<'de>>,
{
    pub fn init_static_symbols(nodes: I) -> Preprocessor<'de, I, StaticSymbolInited> {
        let virtual_registers = (0..=15).map(|r| (Cow::Owned(format!("R{r}")), r));

        let predefined_pointers = [
            (Cow::Borrowed("SP"), 0),
            (Cow::Borrowed("LCL"), 1),
            (Cow::Borrowed("ARG"), 2),
            (Cow::Borrowed("THIS"), 3),
            (Cow::Borrowed("THAT"), 4),
        ]
        .into_iter();

        let i_o_pointers = [
            (Cow::Borrowed("SCREEN"), 16384),
            (Cow::Borrowed("KBD"), 24576),
        ]
        .into_iter();

        let symbol_table = virtual_registers
            .chain(predefined_pointers)
            .chain(i_o_pointers)
            .collect();

        Preprocessor {
            nodes,
            symbol_table,
            next_free_memory_address: 16,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<'de, I> Preprocessor<'de, I, StaticSymbolInited>
where
    I: IntoIterator<Item = Node<'de>> + FromIterator<Node<'de>>,
{
    pub fn extract_source_symbols(self) -> Preprocessor<'de, I, SymbolExtractedState> {
        let nodes = self.nodes;
        let mut symbol_table = self.symbol_table;
        let mut next_free_memory_address = self.next_free_memory_address;

        let nodes = Preprocessor::extract_label_symbols(nodes, &mut symbol_table);
        let nodes = Preprocessor::extract_variable_symbols(
            nodes,
            &mut symbol_table,
            &mut next_free_memory_address,
        );

        Preprocessor {
            nodes,
            symbol_table,
            next_free_memory_address,
            _marker: std::marker::PhantomData,
        }
    }

    fn extract_label_symbols(nodes: I, symbol_table: &mut SymbolTable<'de>) -> I {
        nodes
            .into_iter()
            .fold(vec![], |mut nodes, node| {
                match node {
                    Node::Label { name, .. } => {
                        let len = nodes.len();

                        symbol_table.insert(name.lexeme.clone(), len as Address);
                    }
                    Node::Instruction(_) => {
                        nodes.push(node);
                    }
                };

                nodes
            })
            .into_iter()
            .collect()
    }

    fn extract_variable_symbols(
        nodes: I,
        symbol_table: &mut SymbolTable<'de>,
        next_free_memory_address: &mut Address,
    ) -> I {
        nodes
            .into_iter()
            .map(|node| {
                match &node {
                    Node::Instruction(instruction) => match instruction {
                        crate::parser::Instruction::A { token, .. }
                            if matches!(token.token_type, TokenType::IDENTIFIER) =>
                        {
                            if !symbol_table.contains_key(token.lexeme.as_ref()) {
                                symbol_table
                                    .insert(token.lexeme.clone(), *next_free_memory_address);
                                *next_free_memory_address += 1;
                            }
                        }
                        _ => {}
                    },
                    Node::Label { .. } => unreachable!(),
                }

                node
            })
            .collect()
    }
}

impl<'de, I> Preprocessor<'de, I, SymbolExtractedState>
where
    I: IntoIterator<Item = Node<'de>> + FromIterator<Node<'de>>,
{
    pub fn replace_source_symbols<U>(self) -> U
    where
        U: IntoIterator<Item = Node<'de>> + FromIterator<Node<'de>>,
    {
        let nodes = self.nodes;
        let symbol_table = self.symbol_table;

        nodes
            .into_iter()
            .map(|mut node| match &mut node {
                Node::Instruction(instruction) => match instruction {
                    crate::parser::Instruction::A { token, .. }
                        if matches!(token.token_type, TokenType::IDENTIFIER) =>
                    {
                        assert!(symbol_table.contains_key(&token.lexeme));

                        let &symbol_table_value = symbol_table
                            .get(&token.lexeme)
                            .expect("Symbols should have been extracted in a previous step");

                        *token = Token::new(
                            TokenType::NUMBER(symbol_table_value),
                            Cow::Owned(format!("{symbol_table_value}")),
                            token.line,
                        );

                        node
                    }
                    _ => node,
                },
                Node::Label { .. } => unreachable!(),
            })
            .collect()
    }
}
