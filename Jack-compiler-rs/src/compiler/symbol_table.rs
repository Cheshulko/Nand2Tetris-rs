use crate::{parser::Type, tokenizer::Identifier};

use std::collections::HashMap;
use std::hash::{Hash, Hasher};

#[allow(unused)]
#[derive(Debug)]
pub(super) struct InitialState;

#[derive(Debug)]
pub(super) struct ClassSymbolTableState;

#[derive(Debug)]
pub(super) struct SubroutineSymbolTableState;

type Key<'de> = &'de Identifier<'de>;
type Value<'de> = (&'de Type<'de>, usize);

pub(super) struct SymbolTable<'de, State> {
    static_table: Option<HashMap<Key<'de>, Value<'de>>>,
    field_table: Option<HashMap<Key<'de>, Value<'de>>>,

    argument_table: Option<HashMap<Key<'de>, Value<'de>>>,
    var_table: Option<HashMap<Key<'de>, Value<'de>>>,

    _marker: std::marker::PhantomData<State>,
}

impl<'de> SymbolTable<'de, InitialState> {
    pub(super) fn new_class_symbol_table() -> SymbolTable<'de, ClassSymbolTableState> {
        SymbolTable::<'de, ClassSymbolTableState> {
            static_table: Some(HashMap::new()),
            field_table: Some(HashMap::new()),

            argument_table: None,
            var_table: None,

            _marker: std::marker::PhantomData,
        }
    }

    pub(super) fn new_subroutine_symbol_table() -> SymbolTable<'de, SubroutineSymbolTableState> {
        SymbolTable::<'de, SubroutineSymbolTableState> {
            static_table: None,
            field_table: None,

            argument_table: Some(HashMap::new()),
            var_table: Some(HashMap::new()),

            _marker: std::marker::PhantomData,
        }
    }
}

impl<'de> SymbolTable<'de, ClassSymbolTableState> {
    pub(super) fn insert_field(&mut self, key: Key<'de>, value: &'de Type<'de>) {
        let field_table = self.field_table.as_mut().expect("Class symbol table");

        let index = field_table.len();
        field_table.insert(key, (value, index));
    }

    pub(super) fn get_field(&self, key: Key<'de>) -> Option<&Value<'de>> {
        let field_table = self.field_table.as_ref().expect("Class symbol table");

        field_table.get(key)
    }

    pub(super) fn get_fields_cnt(&self) -> usize {
        let field_table = self.field_table.as_ref().expect("Class symbol table");

        field_table.len()
    }

    pub(super) fn insert_static(&mut self, key: Key<'de>, value: &'de Type<'de>) {
        let static_table = self.static_table.as_mut().expect("Class symbol table");

        let index = static_table.len();
        static_table.insert(key, (value, index));
    }

    pub(super) fn get_static(&self, key: Key<'de>) -> Option<&Value<'de>> {
        let static_table = self.static_table.as_ref().expect("Class symbol table");

        static_table.get(key)
    }
}

impl<'de> SymbolTable<'de, SubroutineSymbolTableState> {
    pub(super) fn insert_var(&mut self, key: Key<'de>, value: &'de Type<'de>) {
        let var_table = self.var_table.as_mut().expect("Subroutine symbol table");

        let index = var_table.len();
        var_table.insert(key, (value, index));
    }

    pub(super) fn get_var(&self, key: Key<'de>) -> Option<&Value<'de>> {
        let var_table = self.var_table.as_ref().expect("Subroutine symbol table");

        var_table.get(key)
    }

    pub(super) fn insert_argument(&mut self, key: Key<'de>, value: &'de Type<'de>) {
        let argument_table = self
            .argument_table
            .as_mut()
            .expect("Subroutine symbol table");

        let index = argument_table.len();
        argument_table.insert(key, (value, index));
    }

    pub(super) fn get_argument(&self, key: Key<'de>) -> Option<&Value<'de>> {
        let argument_table = self
            .argument_table
            .as_ref()
            .expect("Subroutine symbol table");

        argument_table.get(key)
    }
}

impl Hash for Identifier<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}
