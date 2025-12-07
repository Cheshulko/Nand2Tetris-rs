use crate::{
    compiler::{
        subroutine_compiler::SubroutineCompiler,
        symbol_table::{ClassSymbolTableState, SymbolTable},
    },
    parser::{Class, ClassVarDec, ClassVarDecKind, Type},
    tokenizer::Identifier,
};

pub(super) struct ClassCompiler<'de> {
    class: &'de Class<'de>,
    label_index: usize,
    symbol_table: SymbolTable<'de, ClassSymbolTableState>,
    output: Vec<String>,
}

impl<'de> ClassCompiler<'de> {
    pub fn compile(class: &'de Class<'de>) -> anyhow::Result<Vec<String>> {
        let mut compiler = Self {
            class,
            label_index: 0,
            symbol_table: SymbolTable::new_class_symbol_table(),
            output: vec![],
        };

        for class_var_dec in class.class_var_decs.iter() {
            compiler.compile_class_var_dec(class_var_dec)?;
        }

        for subroutine_dec in class.subroutine_decs.iter() {
            let subroutine_instructions =
                SubroutineCompiler::compile(&mut compiler, subroutine_dec)?;
            compiler.output.extend(subroutine_instructions);
        }

        Ok(compiler.output)
    }

    pub(super) fn get_field(&self, key: &'de Identifier<'de>) -> Option<&(&'de Type<'de>, usize)> {
        self.symbol_table.get_field(key)
    }

    pub(super) fn get_fields_cnt(&self) -> usize {
        self.symbol_table.get_fields_cnt()
    }

    pub(super) fn get_static(&self, key: &'de Identifier<'de>) -> Option<&(&'de Type<'de>, usize)> {
        self.symbol_table.get_static(key)
    }

    pub(super) fn get_class(&self) -> &Class<'de> {
        self.class
    }

    pub(super) fn create_new_label(&mut self) -> String {
        let label = format!("{}_{}", self.class.class_name.0, self.label_index);
        self.label_index += 1;

        label
    }

    fn compile_class_var_dec(&mut self, class_var_dec: &'de ClassVarDec<'_>) -> anyhow::Result<()> {
        match &class_var_dec.class_var_dec_kind {
            ClassVarDecKind::Static => {
                for var_name in class_var_dec.var_names.iter() {
                    self.symbol_table
                        .insert_static(var_name, &class_var_dec.class_var_dec_type);
                }

                Ok(())
            }
            ClassVarDecKind::Field => {
                for var_name in class_var_dec.var_names.iter() {
                    self.symbol_table
                        .insert_field(var_name, &class_var_dec.class_var_dec_type);
                }

                Ok(())
            }
        }
    }
}
