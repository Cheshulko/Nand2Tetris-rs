# Nand2Tetris (Rust)

[Nand2Tetris](https://www.nand2tetris.org/) cource

## Table of Contents

- [ ] Jack language compiler
- [x] [**Jack language VM translator**](https://github.com/Cheshulko/Nand2Tetris-rs/tree/main/Jack-vm-translator-rs). A lightweight Rust-based tool that converts Nand2Tetris-style .vm files into Hack assembly .asm files. It implements a full lexing, parsing, and translation pipeline, and supports both individual files and entire directories in one run. Debug output (tokens, AST) can be optionally enabled for introspection.
- [x] [**Hack language assembler**](https://github.com/Cheshulko/Nand2Tetris-rs/tree/main/Hack-assembler-rs). A compact Rust implementation of a Hack assembly → binary translator. It parses .asm files, resolves symbols and labels, and emits .hack (and optional raw .hack.bin) outputs, with debug flags for tokens, AST, and the symbol table.


## Acknowledgments & References

- [**Nand2Tetris Project**](https://www.nand2tetris.org/) — the original Hack platform specification  
- [**Coursera: Build a Modern Computer from First Principles, Part I**](https://www.coursera.org/learn/build-a-computer/) — course (I) these projects are inspired by  
- [**Coursera: Build a Modern Computer from First Principles, Part II**](https://www.coursera.org/learn/nand2tetris2) — course (II) these projects are inspired by  
