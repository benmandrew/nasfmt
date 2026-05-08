# nasfmt

Code formatter for NASM x86-64 assembly.

## Install

```sh
$ cargo install nasfmt
```

Requires Rust 1.95 or later.

## Usage

Format a file in place:

```sh
$ nasfmt file.s
```

Format multiple files in place:

```sh
$ nasfmt *.s
```

Check formatting without modifying files (exits 1 with a diff if changes are needed):

```sh
$ nasfmt --check file.s
```

## What it does

- Normalises mnemonics, registers, and size prefixes to lowercase (`MOV RAX` → `mov rax`)
- Aligns operands and inline comments consistently within each instruction block
- Ensures labels appear on their own line
- Standardises spacing inside memory operands (`[rbp-8]` → `[rbp - 8]`)
- Leaves user-defined symbol names and string literals untouched

## Reference

- [NASM documentation](https://www.nasm.us/doc/)
