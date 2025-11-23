# Jack VM Translator (Rust)

*A small Rust implementation of the Nand2Tetris VM translator*

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)  
[![Rust Version](https://img.shields.io/badge/rust-1.70%2B-lightgrey.svg)](https://www.rust-lang.org/)  

## Table of Contents

* [Overview](#overview)
* [Features](#features)
* [Quick Start](#quick-start)
* [Installation](#installation)
* [Configuration](#configuration)
* [Usage / Examples](#usage--examples)
* [CLI Reference](#cli-reference)
* [Development](#development)
* [License](#license)

## Overview

VMTranslator is a small command-line Rust tool that translates Nand2Tetris-style `.vm` files into Hack assembly `.asm`. It includes a lexer (scanner), parser, and translator modules. The translator produces a vector of assembly instructions (strings) which are then written to the output `.asm` file.

The tool can operate on a single `.vm` file or on a directory containing multiple `.vm` files (all `.vm` files will be processed and their resulting assembly appended to the output). Debug output for tokens and AST can be enabled using environment variables.

## Features

* Lexing of VM commands into tokens (`scanner`)
* Parsing tokens into AST nodes (`parser`)
* Translating parsed VM nodes to Hack assembly (`translator`)
* CLI for file/directory input and optional output path
* Optional debug dumps: token list and AST (written to `*_debug` folders)

## Quick Start

Build and run (example translating a single file):

```bash
# build
cargo build --release

# run on a single file
cargo run -- input/BasicTest.vm -o out/BasicTest.asm
```

When giving a directory as `input`, all `.vm` files inside it are translated and appended to the chosen output `.asm`.

## Installation

From the repository root:

```bash
# Build locally
cargo build --release

# Optionally install locally to cargo bin
cargo install --path .
```

OS-specific notes:

* On Unix-like systems, ensure you have `rust` toolchain and `cargo` installed.
* Output file/directory permissions must allow creation and appending (the translator uses `OpenOptions::append(true)`).

## Configuration

The binary recognizes the following environment variables for debug output:

* `DEBUG_TOKENS` — if set, a tokens dump file (`<input_file>_debug/*.tokens`) is created.
* `DEBUG_AST` — if set, an AST dump file (`<input_file>_debug/*.ast`) is created.
* `DEBUG_ALL` — enables both tokens and AST debug dumps.

Files are created under a sibling directory named `<input_filename>_debug`.

Assumption: debug output files are written next to each input file inside a directory named `<file>_debug` (see `create_debug_file` in `src/main.rs`).

---

## Usage / Examples

Below are short, direct usage snippets copied from the project showing the typical pipeline (scanner → parser → translator → write output). Each snippet includes the minimal context so it is standalone.

### 1) Scan source into tokens

```rust
use crate::scanner::Scanner;
let source = std::fs::read_to_string("input/BasicTest.vm")?;
let tokens: Result<Vec<_>, _> = Scanner::new(&source).collect();
```

### 2) Parse tokens into nodes

```rust
use crate::parser::Parser;
let nodes: Result<Vec<_>, _> = Parser::new(tokens.into_iter()).collect();
let nodes = nodes?;
```

### 3) Translate nodes into assembly

```rust
use crate::translator::Translator;
let translator = Translator::new(stem.display().to_string(), nodes);
let instructions = translator.translate();
for instruction in instructions {
    writeln!(&mut output_file, "{}", instruction)?;
}
```

*(The three snippets above are taken from `src/main.rs` and `src/translator.rs` and show the normal processing flow.)*

### 4) Example CLI invocation

Translate one file:

```bash
# from project root
cargo run -- input/SimpleAdd.vm -o out/SimpleAdd.asm
```

Translate all VM files in a directory:

```bash
cargo run -- input/ -o out/AllPrograms.asm
```

Assumption: when a directory is used as input, all `.vm` files found are translated and appended to the same output `.asm` file (the program opens output with append mode).

## CLI Reference

### Command Syntax

```
VMTranslator <input_path> [-o <output_file>]
```

### Arguments

* **`input_path`**
  Path to a `.vm` file or a directory containing multiple `.vm` files.

* **`-o, --output <output_file>`**
  Optional. Path to the resulting `.asm` file.
  If omitted and the input is a file, the output becomes `<input_stem>.asm`.
  If input is a directory, **Assumption:** the output must be explicitly provided.

### Debug Flags (via environment variables)

Use them when running the binary:

```bash
DEBUG_TOKENS=1 cargo run -- input/SimpleAdd.vm
DEBUG_AST=1 cargo run -- input/SimpleAdd.vm
DEBUG_ALL=1 cargo run -- input/SimpleAdd.vm
```

Output debug files are written into automatically created `<file>_debug/` directories.

---

## Development

### Project Layout (simplified)

```
src/
 ├─ main.rs         # CLI, file orchestration
 ├─ scanner.rs      # Tokenizer for .vm source
 ├─ parser.rs       # AST builder from tokens
 └─ translator.rs   # Produces Hack assembly
```

## License

MIT License.
If a LICENSE file is missing, the project is assumed to be MIT unless specified otherwise.