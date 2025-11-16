use std::env;
use std::fs::{File, create_dir_all, read_to_string};
use std::io::Write;
use std::path::Path;

use clap::Parser as _;

use crate::assembler::Assembler;
use crate::parser::Parser;
use crate::preprocessor::Preprocessor;
use crate::scanner::Scanner;

mod assembler;
mod parser;
mod preprocessor;
mod scanner;

const DEBUG_ALL: &str = "DEBUG_ALL";
const DEBUG_TOKENS: &str = "DEBUG_TOKENS";
const DEBUG_AST: &str = "DEBUG_AST";
const DEBUG_SYMBOL_TABLE: &str = "DEBUG_SYMBOL_TABLE";
const DEBUG_AST_L: &str = "DEBUG_AST_L";

#[derive(clap::Parser)]
#[command(about = "Hack language assembler", long_about = None)]
struct Cli {
    /// Input .asm file
    input: String,

    /// Output .hack file
    #[arg(short = 'o', long, help = ".hack output")]
    output: String,

    /// Additionally: Output to binary .hack.bin
    #[clap(long)]
    bin: bool,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let input_path = Path::new(&cli.input);
    let output_path = Path::new(&cli.output);
    println!("[->] Input file: {}", input_path.display());
    println!("[<-] Output file: {}", output_path.display());

    // 1. Scanning ..
    let source = read_to_string(&input_path)?;
    let tokens: Result<Vec<_>, _> = Scanner::new(&source).into_iter().collect();
    let tokens = tokens?;
    if test_debug(DEBUG_TOKENS) {
        let mut debug_output_file = create_debug_file(&output_path, "tokens")?;

        for token in tokens.iter() {
            let _ = writeln!(&mut debug_output_file, "{token:#?}");
        }
    }

    // 2. Parsing ..
    let nodes: Result<Vec<_>, _> = Parser::new(tokens.into_iter()).collect();
    let nodes = nodes?;
    if test_debug(DEBUG_AST) {
        let mut debug_output_file = create_debug_file(&output_path, "ast")?;

        for node in nodes.iter() {
            writeln!(&mut debug_output_file, "{node:#?}")?;
        }
    }

    // 3. Preprocessing ..
    let preprocessor = Preprocessor::init_static_symbols(nodes).extract_source_symbols();
    if test_debug(DEBUG_SYMBOL_TABLE) {
        let mut debug_output_file = create_debug_file(&output_path, "symbol_table")?;
        let symbol_table = preprocessor.symbol_table();

        writeln!(&mut debug_output_file, "{symbol_table:#?}")?;
    }

    let nodes: Vec<_> = preprocessor.replace_source_symbols();
    if test_debug(DEBUG_AST_L) {
        let mut debug_output_file = create_debug_file(&output_path, "ast_L")?;

        for node in nodes.iter() {
            writeln!(&mut debug_output_file, "{node:#?}")?;
        }
    }

    // 4. Assembling ..
    let assembler = Assembler::new(nodes).assemble();
    let mut output_file = File::create(&output_path)?;
    for (i, x) in assembler.iter().enumerate() {
        write!(&mut output_file, "{:016b}", x)?;

        if i != assembler.len() - 1 {
            write!(&mut output_file, "\n")?;
        }
    }

    if cli.bin {
        let mut output_file_binary = File::create(format!("{}.bin", output_path.display()))?;

        for x in assembler.iter() {
            output_file_binary.write_all(&x.to_be_bytes())?;
        }
    }

    Ok(())
}

fn test_debug<S>(s: S) -> bool
where
    S: AsRef<str>,
{
    env::var(s.as_ref()).is_ok() || env::var(DEBUG_ALL).is_ok()
}

fn create_debug_file<P, S>(output_path: P, suffix: S) -> anyhow::Result<File>
where
    P: AsRef<Path>,
    S: AsRef<str>,
{
    let parent_output_path = output_path
        .as_ref()
        .parent()
        .unwrap_or_else(|| Path::new("."));

    let file_name = output_path.as_ref().file_name().expect("").display();

    let debug_dir = parent_output_path.join(format!("{file_name}_debug",));

    create_dir_all(&debug_dir)?;

    let path = debug_dir.join(format!("{}.{}", file_name, suffix.as_ref()));
    let debug_output_file = File::create(path)?;

    Ok(debug_output_file)
}
