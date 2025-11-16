# Hack-Assembler (Rust)

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)  
[![Rust Version](https://img.shields.io/badge/rust-1.70%2B-lightgrey.svg)](https://www.rust-lang.org/)  

A Hack assembly language → binary translator (assembler) written in Rust.  
Inspired by the **Nand2Tetris** / “From Nand to Tetris” course and the Coursera “Build a Modern Computer from First Principles” track, but not a line-for-line clone: it adopts the same Hack language principles but with certain divergences and Rust-specific design decisions.  

---

## Table of Contents

- [Features](#features)  
- [Getting Started](#getting-started)  
  - [Prerequisites](#prerequisites)  
  - [Install / Build](#install--build)  
  - [Usage](#usage)  
- [Language / Specification Support](#language--specification-support)  
- [Architecture Overview](#architecture-overview)  
- [Debug Output Flags](#debug-output-flags)
- [Examples](#examples)  
- [Testing](#testing)  
- [License](#license)  
- [Acknowledgments & References](#acknowledgments--references)  

---

## Features

- Parses Hack assembly (`.asm`) files and outputs the corresponding binary code (`.hack`)  
- Handles symbol resolution, labels (e.g. `(LOOP)`), predefined symbols, variables  
- Supports comments, whitespace, and basic error reporting  
- Pure Rust implementation with no external dependencies (beyond typical crates)  
- Easily extensible for further hacks or teaching uses  
- **Debugging support**: Offers token, AST, and symbol table outputs via environment variables.
- **Binary output**: Use the `--bin` flag to generate a raw binary `.hack.bin` file alongside the standard `.hack` file.
---

## Getting Started

### Prerequisites

- Rust (stable channel, version ≥ 1.70)  
- `cargo` (comes with Rust)  
- Basic knowledge of the Hack assembly language (as defined in the Nand2Tetris course)  

### Install / Build

Clone the repo and build:

```bash
git clone https://github.com/Cheshulko/Hack-assembler-rs.git
cd Hack-assembler-rs
cargo build --release
```

### Usage
```bash
./hack-assembler-rs input/Max.asm -o output/Max.hack
```

Alternatively, you may run it via `cargo run`:
```bash
cargo run -- input/Max.asm -o output/Max.hack
```

## Language / Specification Support

This assembler supports the **core Hack assembly language** from the Nand2Tetris curriculum:

- **A-instructions:**  
  `@value` — where `value` is a decimal constant or a symbolic label.
- **C-instructions:**  
  `dest=comp;jump` — standard computation and branching syntax.
- **Labels:**  
  Pseudo-commands like `(LABEL)` used for marking addresses.
- **Symbols:**  
  Predefined symbols such as `SP`, `LCL`, `ARG`, `THIS`, `THAT`, `R0–R15`, `SCREEN`, and `KBD`.
- **Variable memory allocation:**  
  User-defined symbols are automatically assigned starting at RAM address `16`.
- **Comments and whitespace:**  
  Fully supports `// comment` lines and ignores empty or indented lines.

> **Note:** This project is not a direct replica of the official Hack assembler; some behavior (e.g., whitespace handling, symbol resolution, or error messages) may differ slightly for educational or Rust-idiomatic reasons.

---

## Architecture Overview

1. **First pass** — Parses all lines, collecting label definitions `(LABEL)` and mapping them to instruction addresses.  
2. **Second pass** — Translates each instruction (`A` or `C`) into a 16-bit binary string, resolving symbols and variable addresses.  
3. **Output stage** — Writes the resulting machine code into a `.hack` file, one instruction per line.

---
## Debug Output Flags

The assembler supports several environment variables to enable debug output at different stages:

- `DEBUG_ALL`: Enables all debug outputs.  
- `DEBUG_TOKENS`: Outputs the tokenized representation of the input assembly code.  
- `DEBUG_AST`: Outputs the Abstract Syntax Tree (AST).  
- `DEBUG_SYMBOL_TABLE`: Outputs the symbol table after preprocessing.  
- `DEBUG_AST_L`: Outputs the final AST after symbol replacement.

To enable a specific debug output, set the corresponding environment variable. For example:

```bash
DEBUG_TOKENS=1 /hack-assembler-rs input/Max.asm -o output/Max.hack
```

This will output the tokenized representation of the input assembly code.

---


## Examples

Below are example runs of the assembler using the `Add` and `Max` sample programs included in the repository.

### Example: `Add`

**Input (`input/Add.asm`):**

```asm
// This file is part of www.nand2tetris.org
// and the book "The Elements of Computing Systems"
// by Nisan and Schocken, MIT Press.
// File name: projects/06/add/Add.asm

// Computes 1 + 1 and stores the result in R0

@1
D = A
@1
D = D + A
@0
M = D
```

**Command:**

```bash
./hack-assembler-rs input/Add.asm -o output/Add.hack
```

**Output (`output/Add.hack`):**

```text
0000000000000001
1110110000010000
0000000000000001
1110000010010000
0000000000000000
1110001100001000
```

---

### Example: `Max`

**Input (`input/Max.asm`):**

```asm
// This file is part of www.nand2tetris.org
// and the book "The Elements of Computing Systems"
// by Nisan and Schocken, MIT Press.
// File name: projects/06/max/Max.asm

// Computes R2 = max(R0, R1)

@R0
D=M              // D = first number
@R1
D=D-M            // D = first number - second number
@OUTPUT_FIRST
D;JGT            // if D>0 goto OUTPUT_FIRST
@R1
D=M
@R2
M=D
@END
0;JMP
(OUTPUT_FIRST)
@R0
D=M
@R2
M=D
(END)
```

**Command:**

```bash
./hack-assembler-rs input/Max.asm -o output/Max.hack
```

**Output (`output/Max.hack`):**

```text
0000000000000000
1111110000010000
0000000000000001
1111010011010000
0000000000001010
1110001100000001
0000000000000001
1111110000010000
0000000000000010
1110001100001000
0000000000001100
1110101010000111
0000000000000000
1111110000010000
0000000000000010
1110001100001000
0000000000001110
```

---

## Testing

Run the unit and integration tests using Cargo:

```bash
cargo test
```

Tests may include:

- [x] Parsing of individual A- and C-instructions  
- [ ] Validation of symbol resolution  
- [ ] End-to-end `.asm` → `.hack` translation  
- [ ] Regression comparisons against expected outputs (“golden files”)

---

## License

This project is licensed under the **MIT License** — see the [LICENSE](LICENSE) file for details.

---

## Acknowledgments & References

- [**Nand2Tetris Project**](https://www.nand2tetris.org/) — the original Hack platform specification  
- [**Coursera: Build a Modern Computer from First Principles**](https://www.coursera.org/learn/build-a-computer/) — course this project is inspired by  