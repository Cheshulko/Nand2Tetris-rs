use clap::Parser as _;
use std::env;
use std::ffi::OsString;
use std::fs::{create_dir_all, File, OpenOptions};
use std::io::Write;
use std::str::FromStr;
use std::{
    fs::read_to_string,
    path::{Path, PathBuf},
};

mod parser;
mod scanner;
mod translator;

use crate::parser::Parser;
use crate::scanner::Scanner;
use crate::translator::Translator;

const DEBUG_ALL: &str = "DEBUG_ALL";
const DEBUG_TOKENS: &str = "DEBUG_TOKENS";
const DEBUG_AST: &str = "DEBUG_AST";

const VM_EXT: &str = "vm";

#[derive(clap::Parser)]
#[command(about = "Jack language VM translator", long_about = None)]
struct Cli {
    /// Input .vm file or directory
    input: PathBuf,

    /// Output .asm file
    #[arg(short = 'o', long, help = ".asm output")]
    output: Option<PathBuf>,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let input_path = &cli.input;
    let output_path = &cli.output.unwrap_or_else(|| default_output(&cli.input));
    println!("[->] Input: {}", input_path.display());
    println!("[<-] Output: {}", output_path.display());

    if input_path.is_dir() {
        for entry in std::fs::read_dir(input_path)? {
            let path = entry?.path();
            if path.is_file() {
                if let Some(e) = path.extension().and_then(|s| s.to_str()) {
                    if e.eq_ignore_ascii_case(VM_EXT) {
                        let source = read_to_string(&path)?;
                        let _ = handle_file(source, &path, output_path)?;
                    }
                }
            }
        }

        return Ok(());
    } else {
        let source = read_to_string(&input_path)?;

        return handle_file(source, input_path, output_path);
    }
}

fn handle_file<P>(source: String, input_file_path: P, output_path: P) -> anyhow::Result<()>
where
    P: AsRef<Path>,
{
    println!(
        "[->] Input file path: {}",
        input_file_path.as_ref().display()
    );

    // 1. Scanning ..
    let tokens: Result<Vec<_>, _> = Scanner::new(&source).into_iter().collect();
    let tokens = tokens?;
    if test_debug(DEBUG_TOKENS) {
        let mut debug_output_file = create_debug_file(&input_file_path, "tokens")?;

        for token in tokens.iter() {
            let _ = writeln!(&mut debug_output_file, "{token:#?}");
        }
    }

    // 2. Parsing ..
    let nodes: Result<Vec<_>, _> = Parser::new(tokens.into_iter()).collect();
    let nodes = nodes?;
    if test_debug(DEBUG_AST) {
        let mut debug_output_file = create_debug_file(&input_file_path, "ast")?;

        for node in nodes.iter() {
            writeln!(&mut debug_output_file, "{node:#?}")?;
        }
    }

    // 2. Translating ..
    let stem = filename(input_file_path.as_ref());
    let translator = Translator::new(stem.display().to_string(), nodes);
    let instructions = translator.translate();

    let mut output_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(output_path)?;

    for instruction in instructions {
        writeln!(&mut output_file, "{}", instruction)?;
    }

    Ok(())
}

fn filename(input: &Path) -> OsString {
    input
        .file_stem()
        .or_else(|| input.file_name())
        .unwrap_or_else(|| input.as_os_str())
        .to_os_string()
}

fn default_output(input: &Path) -> PathBuf {
    let name = filename(input);

    if input.is_dir() {
        input.join(name).with_extension("asm")
    } else {
        input.with_file_name(name).with_extension("asm")
    }
}

fn create_debug_file<P, S>(path: P, suffix: S) -> anyhow::Result<File>
where
    P: AsRef<Path>,
    S: AsRef<str>,
{
    let parent_path = path.as_ref().parent().unwrap_or_else(|| Path::new("."));
    let file_name = path.as_ref().file_name().expect("").display();
    let debug_dir = parent_path.join(format!("{file_name}_debug",));

    create_dir_all(&debug_dir)?;

    let path = debug_dir.join(format!("{}.{}", file_name, suffix.as_ref()));
    let debug_output_file = File::create(path)?;

    Ok(debug_output_file)
}

fn test_debug<S>(s: S) -> bool
where
    S: AsRef<str>,
{
    env::var(s.as_ref()).is_ok() || env::var(DEBUG_ALL).is_ok()
}
