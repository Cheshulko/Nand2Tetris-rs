use std::fs::OpenOptions;
use std::io::Write;
use std::{
    ffi::OsString,
    fs::read_to_string,
    path::{Path, PathBuf},
};

use clap::Parser as _;

use crate::compiler::Compiler;
use crate::parser::Parser;
use crate::tokenizer::{Token, Tokenizer};

mod compiler;
mod parser;
#[cfg(feature = "xml")]
mod parser_xml;
mod tokenizer;
#[cfg(feature = "xml")]
mod tokenizer_xml;

const JACK_EXT: &str = "jack";

#[derive(clap::Parser)]
#[command(about = "Jack language compiler", long_about = None)]
struct Cli {
    /// Input .jack file or directory
    input: PathBuf,
}

struct Tokens<'de> {
    pub tokens: Vec<Token<'de>>,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let input_path = &cli.input;
    println!("[->] Input: {}", input_path.display());

    if input_path.is_dir() {
        for entry in std::fs::read_dir(input_path)? {
            let path = entry?.path();
            if path.is_file() {
                if let Some(e) = path.extension().and_then(|s| s.to_str()) {
                    if e.eq_ignore_ascii_case(JACK_EXT) {
                        let source = read_to_string(&path)?;
                        let output_path_t = default_output(&path, "T", "xml");
                        let output_path = default_output(&path, "", "xml");
                        let o = default_output(&path, "", "vm");

                        let _ = handle_file(source, &path, &output_path_t, &output_path, &o)?;
                    }
                }
            }
        }

        return Ok(());
    } else {
        let source = read_to_string(&input_path)?;
        let output_path_t = default_output(&cli.input, "T", "xml");
        let output_path = default_output(&cli.input, "", "xml");
        let o = default_output(&cli.input, "", "vm");

        return handle_file(source, input_path, &output_path_t, &output_path, &o);
    }
}

fn handle_file<P>(
    source: String,
    input_file_path: P,
    output_path_t: P,
    output_path: P,
    o: P,
) -> anyhow::Result<()>
where
    P: AsRef<Path>,
{
    println!(
        "[->] Input file path: {}",
        input_file_path.as_ref().display()
    );

    // 1. Scanning ..
    let tokens: Result<Vec<_>, _> = Tokenizer::new(&source).into_iter().collect();
    let tokens = tokens?;
    let tokens = Tokens { tokens };

    #[cfg(feature = "xml")]
    {
        use quick_xml::se::to_string;
        use std::fs::File;

        let xml = to_string(&tokens)?;
        let mut f = File::create(output_path_t)?;
        writeln!(&mut f, "{}\n", xml)?;
    }

    // 2. Parsing ..
    let nodes: Result<Vec<_>, _> = Parser::new(tokens.tokens.into_iter()).collect();
    let nodes = nodes?;

    assert!(nodes.len() == 1);
    #[cfg(feature = "xml")]
    {
        use quick_xml::se::Serializer;
        use serde::Serialize;
        use std::fs::File;

        for node in nodes.iter() {
            let mut output = String::new();
            let mut ser = Serializer::new(&mut output);
            ser.indent(' ', 4);

            node.serialize(ser)?;

            let mut f = File::create(&output_path)?;
            writeln!(&mut f, "{}", output)?;
        }
    }

    // 3. Compiling ..
    let mut compiler = Compiler::new(nodes.iter());
    let instructions = compiler.compile();

    let mut output_file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(o)?;

    for (i, instruction) in instructions.iter().enumerate() {
        if i + 1 != instructions.len() {
            writeln!(&mut output_file, "{instruction}")?;
        } else {
            write!(&mut output_file, "{instruction}")?;
        }
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

fn default_output(input: &Path, suf: &str, ext: &str) -> PathBuf {
    let name = format!("{}{suf}", filename(input).display());

    if input.is_dir() {
        input.join(name).with_extension(ext)
    } else {
        input.with_file_name(name).with_extension(ext)
    }
}
