use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::Command;

const EXAMPLES: &[&str] = &["board.s", "hello.s", "main.s", "memory.s", "util.s"];

fn get_example_path(name: &str) -> String {
    format!("tests/resources/{name}")
}

fn run_nasfmt(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_nasfmt"))
        .args(args)
        .output()
        .expect("Failed to run nasfmt")
}

fn format_string(input: &str) -> String {
    let mut tmp = tempfile::Builder::new().suffix(".s").tempfile().unwrap();
    tmp.write_all(input.as_bytes()).unwrap();
    let path = tmp.path().to_str().unwrap().to_string();
    let output = run_nasfmt(&[&path]);
    assert!(output.status.success(), "nasfmt failed on: {input:?}");
    String::from_utf8(output.stdout).unwrap()
}

fn format_string_upper(input: &str) -> String {
    let mut tmp = tempfile::Builder::new().suffix(".s").tempfile().unwrap();
    tmp.write_all(input.as_bytes()).unwrap();
    let path = tmp.path().to_str().unwrap().to_string();
    let output = run_nasfmt(&[&path, "--upper"]);
    assert!(
        output.status.success(),
        "nasfmt --upper failed on: {input:?}"
    );
    String::from_utf8(output.stdout).unwrap()
}

// ─── existing tests ────────────────────────────────────────────────────────

#[test]
fn test_examples_format_clean() {
    for &file in EXAMPLES {
        let path = get_example_path(file);
        assert!(
            Path::new(&path).exists(),
            "Example file {path} does not exist"
        );
        let output = run_nasfmt(&[&path, "--check"]);
        assert!(
            output.status.success(),
            "nasfmt failed or formatting needed for {}\nstdout: {}\nstderr: {}",
            path,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

#[test]
fn test_examples_format_idempotent() {
    for &file in EXAMPLES {
        let path = get_example_path(file);
        let orig = fs::read_to_string(&path).expect("Failed to read example file");
        let output = run_nasfmt(&[&path]);
        assert!(output.status.success(), "nasfmt failed for {path}");
        let formatted = String::from_utf8_lossy(&output.stdout);
        assert_eq!(
            orig, formatted,
            "nasfmt output is not idempotent for {path}"
        );
    }
}

// ─── exit-code tests ───────────────────────────────────────────────────────

#[test]
fn test_nonexistent_file_exits_2() {
    let output = run_nasfmt(&["/nonexistent/path/to/file.s"]);
    assert_eq!(output.status.code(), Some(2));
}

#[test]
fn test_check_clean_file_exits_0() {
    let output = run_nasfmt(&["tests/resources/memory.s", "--check"]);
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn test_check_dirty_file_exits_1() {
    let mut tmp = tempfile::Builder::new().suffix(".s").tempfile().unwrap();
    tmp.write_all(b"    MOV RAX, RBX\n    RET\n").unwrap();
    let path = tmp.path().to_str().unwrap().to_string();
    let check_output = run_nasfmt(&[&path, "--check"]);
    assert_eq!(
        check_output.status.code(),
        Some(1),
        "expected exit 1 for dirty file, stdout: {}",
        String::from_utf8_lossy(&check_output.stdout)
    );
    // stdout should contain a diff
    let stdout = String::from_utf8_lossy(&check_output.stdout);
    assert!(
        stdout.contains('+') || stdout.contains('-'),
        "diff output expected"
    );
}

// ─── formatting behaviour tests ────────────────────────────────────────────

#[test]
fn test_format_empty_file() {
    assert_eq!(format_string(""), "");
}

#[test]
fn test_format_single_blank_line() {
    assert_eq!(format_string("\n"), "\n");
}

#[test]
fn test_format_normalizes_mnemonic_case() {
    let result = format_string("    RET\n");
    assert_eq!(result, "    ret\n");
}

#[test]
fn test_format_normalizes_register_case() {
    let result = format_string("    MOV RAX, RBX\n    RET\n");
    assert_eq!(result, "    mov rax, rbx\n    ret\n");
}

#[test]
fn test_format_normalizes_size_prefix_case() {
    let result = format_string("    mov BYTE [rax], 0\n");
    assert_eq!(result, "    mov byte [rax], 0\n");
}

#[test]
fn test_format_preserves_user_symbol_case() {
    // MyLabel is not a register, must preserve case.
    // Single-instruction block: mnemonic_width = round_up_4(4+1) = 8
    let result = format_string("    call MyLabel\n");
    assert_eq!(result, "    call    MyLabel\n");
}

#[test]
fn test_format_section_directive_not_indented() {
    let result = format_string("section .text\n");
    assert_eq!(result, "section .text\n");
}

#[test]
fn test_format_bits_directive_lowercased() {
    let result = format_string("BITS 64\n");
    assert_eq!(result, "bits 64\n");
}

#[test]
fn test_format_global_directive() {
    let result = format_string("global main\n");
    assert_eq!(result, "global main\n");
}

#[test]
fn test_format_extern_directive() {
    let result = format_string("extern printf\n");
    assert_eq!(result, "extern printf\n");
}

#[test]
fn test_format_code_label_splits_to_own_line() {
    // "main: mov rax, 0" → label on own line, body indented below
    let result = format_string("main: mov rax, 0\n");
    assert_eq!(result, "main:\n    mov rax, 0\n");
}

#[test]
fn test_format_bare_code_label() {
    let result = format_string("main:\n");
    assert_eq!(result, "main:\n");
}

#[test]
fn test_format_code_label_with_comment() {
    let result = format_string("main: ; entry point\n");
    assert_eq!(result, "main:    ; entry point\n");
}

#[test]
fn test_format_aligns_instruction_block() {
    // push (4) is longest → mnemonic_width = round_up_4(5) = 8
    let input = "    push rbx\n    mov rax, 0\n    ret\n";
    let expected = "    push    rbx\n    mov     rax, 0\n    ret\n";
    assert_eq!(format_string(input), expected);
}

#[test]
fn test_format_aligns_comments_in_block() {
    // Both len-3 mnemonics → mnemonic_width=4; max_ops=len("eax, eax")=8
    // comment_col = 4 + 0 + 4 + 8 + 4 = 20
    let input = "    ret ; end\n    xor eax, eax ; zero\n";
    let result = format_string(input);
    for line in result.lines() {
        if line.contains(';') {
            let col = line.find(';').unwrap();
            assert_eq!(col, 20, "comment not at col 20: {line:?}");
        }
    }
}

#[test]
fn test_format_blank_lines_preserved() {
    // Blank lines split blocks but alignment is computed across the whole file
    let input = "    push rbx\n\n    ret\n";
    let result = format_string(input);
    assert!(result.contains("\n\n"), "blank line should be preserved");
}

#[test]
fn test_format_aligns_mnemonics_across_blank_lines() {
    // push (4 chars) → mnemonic_width=8 for the whole file;
    // mov in the second block must also use width 8
    let input = "    push rbx\n\n    mov rax, 0\n";
    let result = format_string(input);
    assert_eq!(result, "    push    rbx\n\n    mov     rax, 0\n");
}

#[test]
fn test_format_aligns_comments_across_blank_lines() {
    // Both blocks share the same comment_col derived from the widest line in the file
    let input = "    push rbx ; save\n\n    ret ; done\n";
    let result = format_string(input);
    let cols: Vec<usize> = result
        .lines()
        .filter(|l| l.contains(';'))
        .map(|l| l.find(';').unwrap())
        .collect();
    assert!(cols.len() == 2, "expected two commented lines");
    assert_eq!(cols[0], cols[1], "comment columns must match across blocks");
}

#[test]
fn test_format_preserves_preprocessor_directive() {
    let result = format_string("%define SIZE 10\n");
    assert_eq!(result, "%define SIZE 10\n");
}

#[test]
fn test_format_preprocessor_resets_block() {
    // Preprocessor should flush any open block
    let input = "    mov rax, 0\n%define X 1\n    ret\n";
    let result = format_string(input);
    assert!(result.contains("%define X 1\n"));
}

#[test]
fn test_format_comment_only_line() {
    let result = format_string("; This is a comment\n");
    assert_eq!(result, "; This is a comment\n");
}

#[test]
fn test_format_indented_comment() {
    let result = format_string("    ; indented comment\n");
    assert_eq!(result, "    ; indented comment\n");
}

#[test]
fn test_format_bare_semicolon() {
    let result = format_string(";\n");
    assert_eq!(result, ";\n");
}

#[test]
fn test_standalone_comment_before_label_not_indented() {
    // Comment describing a function — should sit at column 0 like the label
    let input = "    ; entry point\nmain:\n    ret\n";
    let result = format_string(input);
    assert!(
        result.starts_with("; entry point\n"),
        "comment before label should have no indent, got: {result:?}"
    );
}

#[test]
fn test_standalone_comment_before_instruction_indented() {
    // Comment describing a block of instructions — should align with the mnemonic
    let input = "main:\n; initialise\n    mov rax, 0\n    ret\n";
    let result = format_string(input);
    assert!(
        result.contains("    ; initialise\n"),
        "comment before instruction should be indented 4 spaces, got: {result:?}"
    );
}

#[test]
fn test_standalone_comment_before_label_across_blank_line() {
    // Blank line between comment and label — comment should still be unindented
    let input = "; describe function\n\nmain:\n    ret\n";
    let result = format_string(input);
    assert!(
        result.starts_with("; describe function\n"),
        "comment before label (across blank) should have no indent, got: {result:?}"
    );
}

#[test]
fn test_inline_comment_continuation_indent_preserved() {
    // The second line of a wrapped inline comment must not be normalised to
    // the mnemonic indent; it should stay aligned with the `;` above it.
    let col = " ".repeat(38);
    let input = format!(
        "    cmp     r12, 0                    ; Only notify of invalid input the first\n{col}; time\n    jne     somewhere\n",
    );
    let result = format_string(&input);
    let cont = result.lines().nth(1).unwrap_or("");
    assert_eq!(
        cont,
        format!("{col}; time"),
        "continuation indent should be preserved at column 38"
    );
}

#[test]
fn test_standalone_comment_before_section_directive_not_indented() {
    let input = "    ; data section\nsection .data\n";
    let result = format_string(input);
    assert!(
        result.starts_with("; data section\n"),
        "comment before section directive should have no indent, got: {result:?}"
    );
}

#[test]
fn test_standalone_comment_before_preprocessor_not_indented() {
    let input = "    ; define size\n%define SIZE 10\n";
    let result = format_string(input);
    assert!(
        result.starts_with("; define size\n"),
        "comment before preprocessor should have no indent, got: {result:?}"
    );
}

#[test]
fn test_format_data_block_aligns_labels_and_directives() {
    // msg (3), newline (7) → label_width = round_up_4(8) = 8
    // db (2) → mnemonic_width = round_up_4(3) = 4
    let input = "section .data\n    msg db \"Hello\", 0\n    newline db 10\n";
    let expected = "section .data\n    msg     db  \"Hello\", 0\n    newline db  10\n";
    assert_eq!(format_string(input), expected);
}

#[test]
fn test_format_memory_relative_address_space_preserved() {
    // [rel msg] must NOT collapse to [relmsg]
    let result = format_string("    lea rdi, [rel msg]\n");
    assert_eq!(result, "    lea rdi, [rel msg]\n");
}

#[test]
fn test_format_memory_arithmetic_spacing() {
    // [rbp-8] → [rbp - 8]
    let result = format_string("    mov rax, [rbp-8]\n");
    assert_eq!(result, "    mov rax, [rbp - 8]\n");
}

#[test]
fn test_format_memory_multiply_spacing() {
    let result = format_string("    lea rcx, [rax+rbx*4]\n");
    assert_eq!(result, "    lea rcx, [rax + rbx * 4]\n");
}

#[test]
fn test_format_size_prefix_in_memory_operand() {
    let result = format_string("    mov qword [rbp-8], rdi\n");
    assert_eq!(result, "    mov qword [rbp - 8], rdi\n");
}

#[test]
fn test_format_string_with_semicolon_not_treated_as_comment() {
    let input = "    db \"hello; world\"\n";
    let result = format_string(input);
    // The semicolon inside the string must be preserved; no comment should be appended
    assert_eq!(result, "    db  \"hello; world\"\n");
}

#[test]
fn test_format_operands_separated_by_comma_and_space() {
    let result = format_string("    mov rax,rbx\n");
    // Comma without space → formatted with ", "
    assert_eq!(result, "    mov rax, rbx\n");
}

#[test]
fn test_format_multiple_sections() {
    let input = "section .data\n    x db 0\n\nsection .text\n    ret\n";
    let result = format_string(input);
    assert!(result.contains("section .data\n"));
    assert!(result.contains("section .text\n"));
    assert!(result.contains("    ret\n"));
}

#[test]
fn test_format_section_with_global_and_code() {
    let input = "global main\nextern printf\n\nsection .text\nmain:\n    xor eax, eax\n    ret\n";
    let result = format_string(input);
    assert!(result.contains("global main\n"));
    assert!(result.contains("extern printf\n"));
    assert!(result.contains("main:\n"));
    assert!(result.contains("    xor eax, eax\n"));
    assert!(result.contains("    ret\n"));
}

// ─── --upper flag tests ────────────────────────────────────────────────────

#[test]
fn test_upper_normalises_mnemonic() {
    assert_eq!(format_string_upper("    ret\n"), "    RET\n");
}

#[test]
fn test_upper_normalises_register() {
    assert_eq!(
        format_string_upper("    mov rax, rbx\n"),
        "    MOV RAX, RBX\n"
    );
}

#[test]
fn test_upper_normalises_size_prefix() {
    assert_eq!(
        format_string_upper("    mov byte [rax], 0\n"),
        "    MOV BYTE [RAX], 0\n"
    );
}

#[test]
fn test_upper_preserves_user_symbols() {
    // MyLabel is a user symbol, not a register — must not be touched
    let result = format_string_upper("    call MyLabel\n");
    assert!(result.contains("CALL"), "mnemonic should be uppercased");
    assert!(
        result.contains("MyLabel"),
        "user symbol should be preserved as-is"
    );
}

#[test]
fn test_upper_normalises_memory_operand() {
    assert_eq!(
        format_string_upper("    mov rax, [rbp - 8]\n"),
        "    MOV RAX, [RBP - 8]\n"
    );
}

#[test]
fn test_upper_normalises_section_directive() {
    assert_eq!(format_string_upper("section .text\n"), "SECTION .text\n");
}

#[test]
fn test_upper_normalises_global_directive() {
    assert_eq!(format_string_upper("global main\n"), "GLOBAL main\n");
}

#[test]
fn test_upper_from_mixed_case_input() {
    // Input already has mixed case; --upper should normalise regardless
    assert_eq!(
        format_string_upper("    MOV Rax, RBX\n"),
        "    MOV RAX, RBX\n"
    );
}

#[test]
fn test_upper_check_exits_1_for_lowercase_file() {
    let mut tmp = tempfile::Builder::new().suffix(".s").tempfile().unwrap();
    tmp.write_all(b"    mov rax, rbx\n").unwrap();
    let path = tmp.path().to_str().unwrap().to_string();
    let output = run_nasfmt(&[&path, "--upper", "--check"]);
    assert_eq!(
        output.status.code(),
        Some(1),
        "should exit 1 for non-upper file"
    );
}

#[test]
fn test_upper_check_exits_0_for_uppercase_file() {
    let mut tmp = tempfile::Builder::new().suffix(".s").tempfile().unwrap();
    tmp.write_all(b"    MOV RAX, RBX\n").unwrap();
    let path = tmp.path().to_str().unwrap().to_string();
    let output = run_nasfmt(&[&path, "--upper", "--check"]);
    assert_eq!(
        output.status.code(),
        Some(0),
        "should exit 0 for already-upper file"
    );
}

#[test]
fn test_upper_default_is_lower() {
    // Without --upper, output must be lowercase
    assert_eq!(format_string("    MOV RAX, RBX\n"), "    mov rax, rbx\n");
}
