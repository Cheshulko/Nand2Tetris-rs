# Jack language compiler (Rust)

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)  
[![Rust Version](https://img.shields.io/badge/rust-1.70%2B-lightgrey.svg)](https://www.rust-lang.org/) 

---

## Table of Contents

* [Overview](#overview)
* [Features](#features)
* [Quick Start](#quick-start)
* [Installation](#installation)
* [Configuration](#configuration)

---

## Overview

Jack-compiler-rs is a small Rust implementation of a Jack language compiler (the Jack language from the [Nand2Tetris](https://www.nand2tetris.org/) course). It performs lexical analysis (tokenizer), parsing into an AST, optional XML serialization (for debugging), and compilation to VM instructions. The repository is a CLI tool that accepts a `.jack` file or directory and emits `.vm` output (and optional XML token/parse dumps).

High-level design:

* `tokenizer` — produces tokens from a Jack source string.
* `parser` — converts tokens into an AST (`Class` nodes).
* `compiler` — walks AST nodes and emits VM instructions.
* `main` — CLI glue that wires tokenizer → parser → compiler and writes output.

---

## Features

* CLI that accepts a `.jack` file or directory.
* Tokenizer producing a token stream (supports keywords, symbols, identifiers, constants).
* Parser that builds `Class` AST nodes.
* Compiler that emits VM code (.vm).
* Optional XML serialization of tokens/parse trees behind the `xml` Cargo feature.
* Sample `.jack` programs in the `input/` folder.

---

## Quick Start

Build and run the compiler on a single file:

```bash
# Build & run on a single file (outputs .vm adjacent to input)
cargo run -- path/to/MyClass.jack
```

Compile a folder of `.jack` files (example uses the bundled `input/Test`):

```bash
cargo run -- input/Test
```

To produce XML token/AST dumps (optional feature):

```bash
cargo run --features xml -- input/Test
```

---

## Installation

### Prerequisites

* Rust toolchain (Rust 1.70+ recommended). Install from [https://rustup.rs](https://rustup.rs).

### Build from source

```bash
git clone <repo-url>
cd Jack-compiler-rs
cargo build --release
```

### Run (development)

```bash
# Run with the local source (recommended during development)
cargo run -- path/to/SomeFile.jack
```

---

## Configuration

---
### XML Mode (`--features xml`)

When compiled with `xml` feature, the compiler emits:

* `<file>T.xml` — token stream
* `<file>.xml` — parsed AST dump

No XML is generated unless the feature is explicitly enabled.

Enable with Cargo:

```bash
cargo build --features xml
cargo run -- input/SomeDir
```

No runtime environment variables are required.

---

## Files / important entry points

* `src/main.rs` — CLI and program entry.
* `src/tokenizer.rs` — tokenization logic.
* `src/parser.rs` — parser that produces `Class` AST nodes.
* `src/compiler/*` — compilation modules (class/subroutine compilers & symbol table).
* `input/` — many sample `.jack` programs used as example inputs.

---

# License

MIT License. If a LICENSE file is missing, the project is assumed to be MIT unless specified otherwise.

---