use std::fmt::Display;

use crate::{compiler::class_compiler::ClassCompiler, parser::Class};

mod class_compiler;
mod subroutine_compiler;
pub(super) mod symbol_table;

pub struct Compiler<'de, I: Iterator<Item = &'de Class<'de>>> {
    nodes: I,
    output: Vec<String>,
}

impl<'de, I> Compiler<'de, I>
where
    I: Iterator<Item = &'de Class<'de>> + Clone,
{
    pub fn new(nodes: I) -> Self {
        Self {
            nodes,
            output: vec![],
        }
    }

    pub fn compile(&mut self) -> Vec<String> {
        let mut nodes = self.nodes.clone();

        while let Some(class) = nodes.next() {
            self.compile_class(class).unwrap();
        }

        return self.output.clone();
    }

    fn compile_class(&mut self, class: &Class<'_>) -> anyhow::Result<()> {
        let compiled_class_instructions = ClassCompiler::compile(class)?;

        self.output.extend(compiled_class_instructions);

        Ok(())
    }
}

enum Pad {
    None,
    One,
}

impl Display for Pad {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Pad::None => write!(f, ""),
            Pad::One => write!(f, "    "),
        }
    }
}
