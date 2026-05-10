# nasfmt

[![Crates.io Version](https://img.shields.io/crates/v/nasfmt)](https://crates.io/crates/nasfmt) ![Coverage](docs/coverage.svg)

Code formatter for NASM x86-64 assembly.

## Install

```sh
$ cargo install nasfmt
```

## Usage

Format a file in place:

```sh
$ nasfmt file.s
```

Format multiple files in place:

```sh
$ nasfmt file1.s file2.s file3.s
```

Check formatting without modifying files (exits 1 with a diff if changes are needed):

```diff
$ nasfmt --check tests/resources/main.s
--- tests/resources/main.s
+++ tests/resources/main.s
@@ -103,9 +103,9 @@
     syscall
     cmp     rax, 0
     jle     no_input_connected
-    movzx   rax, byte [input_char] ; Convert ASCII to integer
-    sub   rax, '0'
-    cmp      rax, 1                    ; Check lower bound
+    movzx   rax, byte [input_char]    ; Convert ASCII to integer
+    sub     rax, '0'
+    cmp     rax, 1                    ; Check lower bound
     jl      invalid_input
     cmp     rax, 9                    ; Check upper bound
     jg      invalid_input
```

## What it does

- Normalises mnemonics, registers, and size prefixes to lowercase (`MOV RAX` → `mov rax`), or uppercase if the `--upper` flag is passed
- Aligns operands and comments consistently within each instruction block
- Ensures labels appear on their own line
- Standardises spacing inside memory operands (`[rbp-8]` → `[rbp - 8]`)
- Leaves user-defined symbol names and string literals untouched

## Reference

- [NASM documentation](https://www.nasm.us/doc/)
