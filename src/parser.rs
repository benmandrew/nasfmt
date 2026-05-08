use crate::ast::{Body, Label, Line, SECTION_DIRECTIVES};

const DATA_DIRECTIVES: &[&str] = &[
    "db", "dw", "dd", "dq", "dt", "ddq", "do", "resb", "resw", "resd", "resq", "rest", "resdq",
    "equ", "times", "incbin",
];

pub fn parse(source: &str) -> Vec<Line> {
    source.lines().map(parse_line).collect()
}

fn find_comment_start(s: &str) -> Option<usize> {
    let mut chars = s.char_indices().peekable();
    while let Some((i, c)) = chars.next() {
        match c {
            '"' | '\'' | '`' => {
                let q = c;
                while let Some((_, nc)) = chars.next() {
                    if nc == q {
                        break;
                    }
                    if nc == '\\' && q != '\'' {
                        chars.next();
                    }
                }
            }
            ';' => return Some(i),
            _ => {}
        }
    }
    None
}

fn parse_line(line: &str) -> Line {
    let trimmed_end = line.trim_end();
    if trimmed_end.is_empty() {
        return Line::Blank;
    }
    let trimmed = trimmed_end.trim_start();
    if trimmed.starts_with('%') {
        return Line::Preprocessor(trimmed_end.to_string());
    }
    let (code_raw, comment) = match find_comment_start(trimmed_end) {
        Some(pos) => {
            let raw = &trimmed_end[pos + 1..];
            let text = raw.strip_prefix(' ').unwrap_or(raw).to_string();
            (&trimmed_end[..pos], Some(text))
        }
        None => (trimmed_end, None),
    };
    let code = code_raw.trim_end();
    if code.trim().is_empty() {
        let indent = code_raw.len();
        return match comment {
            Some(text) => Line::CommentOnly { indent, text },
            None => Line::Blank, // bare `;` with no text handled via empty CommentOnly below
        };
    }
    let starts_indented = line.starts_with(|c: char| c.is_whitespace());
    parse_statement(code.trim(), starts_indented, comment)
}

fn parse_statement(code: &str, starts_indented: bool, comment: Option<String>) -> Line {
    let (first, rest) = split_first_token(code);
    let first_lower = first.to_lowercase();
    if starts_indented {
        // Indented data definitions: `    label db ...` / `    label equ ...`
        // Detect by checking if second token is a known data directive.
        let rest_trimmed = rest.trim_start();
        let (second, _) = split_first_token(rest_trimmed);
        if DATA_DIRECTIVES.contains(&second.to_lowercase().as_str()) {
            return Line::Statement {
                label: Some(Label {
                    name: first.to_string(),
                    has_colon: false,
                }),
                body: optional_body(rest_trimmed),
                comment,
            };
        }
        return Line::Statement {
            label: None,
            body: Some(make_body(first, rest)),
            comment,
        };
    }
    if SECTION_DIRECTIVES.contains(&first_lower.as_str()) {
        return Line::Statement {
            label: None,
            body: Some(make_body(first, rest)),
            comment,
        };
    }
    if let Some(name) = first.strip_suffix(':') {
        return Line::Statement {
            label: Some(Label {
                name: name.to_string(),
                has_colon: true,
            }),
            body: optional_body(rest),
            comment,
        };
    }
    Line::Statement {
        label: Some(Label {
            name: first.to_string(),
            has_colon: false,
        }),
        body: optional_body(rest),
        comment,
    }
}

fn split_first_token(s: &str) -> (&str, &str) {
    let end = s.find(|c: char| c.is_whitespace()).unwrap_or(s.len());
    (&s[..end], &s[end..])
}

fn make_body(first: &str, rest: &str) -> Body {
    let mnemonic = first.to_lowercase();
    let operands = if rest.trim().is_empty() {
        vec![]
    } else {
        split_operands(rest.trim())
    };
    Body { mnemonic, operands }
}

fn make_body_str(s: &str) -> Body {
    let (first, rest) = split_first_token(s);
    make_body(first, rest)
}

fn optional_body(rest: &str) -> Option<Body> {
    let trimmed = rest.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(make_body_str(trimmed))
    }
}

fn split_operands(s: &str) -> Vec<String> {
    let mut operands = vec![];
    let mut current = String::new();
    let mut depth = 0usize;
    let mut in_string = false;
    let mut string_char = '"';
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if in_string {
            current.push(c);
            if c == string_char {
                in_string = false;
            } else if c == '\\'
                && string_char != '\''
                && let Some(&next) = chars.peek()
            {
                current.push(next);
                chars.next();
            }
        } else {
            match c {
                '"' | '\'' | '`' => {
                    in_string = true;
                    string_char = c;
                    current.push(c);
                }
                '[' | '(' => {
                    depth += 1;
                    current.push(c);
                }
                ']' | ')' => {
                    depth = depth.saturating_sub(1);
                    current.push(c);
                }
                ',' if depth == 0 => {
                    operands.push(current.trim().to_string());
                    current = String::new();
                }
                _ => current.push(c),
            }
        }
    }
    let last = current.trim().to_string();
    if !last.is_empty() {
        operands.push(last);
    }
    operands
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Line;

    // --- find_comment_start ---

    #[test]
    fn comment_start_no_semicolon() {
        assert_eq!(find_comment_start("mov rax, rbx"), None);
    }

    #[test]
    fn comment_start_simple() {
        assert_eq!(find_comment_start("mov rax, rbx ; comment"), Some(13));
    }

    #[test]
    fn comment_start_at_beginning() {
        assert_eq!(find_comment_start("; comment"), Some(0));
    }

    #[test]
    fn comment_start_semicolon_in_double_quoted_string() {
        assert_eq!(find_comment_start("db \"; not a comment\""), None);
    }

    #[test]
    fn comment_start_semicolon_in_single_quoted_string() {
        assert_eq!(find_comment_start("db '; not'"), None);
    }

    #[test]
    fn comment_start_semicolon_in_backtick_string() {
        assert_eq!(find_comment_start("db `; not`"), None);
    }

    #[test]
    fn comment_start_after_string() {
        assert_eq!(find_comment_start("db \"hi\" ; real"), Some(8));
    }

    #[test]
    fn comment_start_empty_string() {
        assert_eq!(find_comment_start(""), None);
    }

    // --- split_first_token ---

    #[test]
    fn split_first_two_words() {
        assert_eq!(split_first_token("mov rax"), ("mov", " rax"));
    }

    #[test]
    fn split_first_single_word() {
        assert_eq!(split_first_token("ret"), ("ret", ""));
    }

    #[test]
    fn split_first_empty() {
        assert_eq!(split_first_token(""), ("", ""));
    }

    #[test]
    fn split_first_with_tab() {
        assert_eq!(split_first_token("mov\trax"), ("mov", "\trax"));
    }

    // --- split_operands ---

    #[test]
    fn split_operands_empty() {
        assert_eq!(split_operands(""), Vec::<String>::new());
    }

    #[test]
    fn split_operands_single() {
        assert_eq!(split_operands("rax"), vec!["rax"]);
    }

    #[test]
    fn split_operands_two() {
        assert_eq!(split_operands("rax, rbx"), vec!["rax", "rbx"]);
    }

    #[test]
    fn split_operands_three() {
        assert_eq!(split_operands("rax, rbx, rcx"), vec!["rax", "rbx", "rcx"]);
    }

    #[test]
    fn split_operands_no_space_after_comma() {
        assert_eq!(split_operands("rax,rbx"), vec!["rax", "rbx"]);
    }

    #[test]
    fn split_operands_nested_brackets() {
        assert_eq!(
            split_operands("[rax + rbx], rcx"),
            vec!["[rax + rbx]", "rcx"]
        );
    }

    #[test]
    fn split_operands_comma_in_double_quoted_string() {
        assert_eq!(
            split_operands("\"hello, world\", 0"),
            vec!["\"hello, world\"", "0"]
        );
    }

    #[test]
    fn split_operands_comma_in_single_quoted_string() {
        assert_eq!(split_operands("',', 0x0a"), vec!["','", "0x0a"]);
    }

    #[test]
    fn split_operands_whitespace_trimmed() {
        assert_eq!(split_operands("  rax ,  rbx  "), vec!["rax", "rbx"]);
    }

    // --- parse_line ---

    #[test]
    fn parse_blank_line() {
        assert!(matches!(parse_line(""), Line::Blank));
    }

    #[test]
    fn parse_whitespace_only() {
        assert!(matches!(parse_line("   "), Line::Blank));
    }

    #[test]
    fn parse_preprocessor_define() {
        let result = parse_line("%define SIZE 10");
        assert!(matches!(result, Line::Preprocessor(_)));
        if let Line::Preprocessor(s) = result {
            assert_eq!(s, "%define SIZE 10");
        }
    }

    #[test]
    fn parse_preprocessor_preserves_leading_whitespace() {
        // trimmed_end is used, so leading whitespace before '%' is kept
        let result = parse_line("  %ifdef X");
        if let Line::Preprocessor(s) = result {
            assert_eq!(s, "  %ifdef X");
        }
    }

    #[test]
    fn parse_comment_only_unindented() {
        let result = parse_line("; this is a comment");
        if let Line::CommentOnly { indent, text } = result {
            assert_eq!(indent, 0);
            assert_eq!(text, "this is a comment");
        } else {
            panic!("expected CommentOnly");
        }
    }

    #[test]
    fn parse_comment_only_indented() {
        let result = parse_line("    ; indented");
        if let Line::CommentOnly { indent, text } = result {
            assert_eq!(indent, 4);
            assert_eq!(text, "indented");
        } else {
            panic!("expected CommentOnly");
        }
    }

    #[test]
    fn parse_bare_semicolon() {
        let result = parse_line(";");
        if let Line::CommentOnly { indent, text } = result {
            assert_eq!(indent, 0);
            assert_eq!(text, "");
        } else {
            panic!("expected CommentOnly with empty text");
        }
    }

    #[test]
    fn parse_code_label_bare() {
        let result = parse_line("main:");
        if let Line::Statement {
            label: Some(lbl),
            body: None,
            comment: None,
        } = result
        {
            assert_eq!(lbl.name, "main");
            assert!(lbl.has_colon);
        } else {
            panic!("expected Statement with colon label");
        }
    }

    #[test]
    fn parse_code_label_with_body() {
        let result = parse_line("main: mov rax, 0");
        if let Line::Statement {
            label: Some(lbl),
            body: Some(b),
            comment: None,
        } = result
        {
            assert_eq!(lbl.name, "main");
            assert!(lbl.has_colon);
            assert_eq!(b.mnemonic, "mov");
            assert_eq!(b.operands, vec!["rax", "0"]);
        } else {
            panic!("expected Statement with label and body");
        }
    }

    #[test]
    fn parse_code_label_with_comment() {
        let result = parse_line("main: ; entry");
        if let Line::Statement {
            label: Some(lbl),
            body: None,
            comment: Some(c),
        } = result
        {
            assert_eq!(lbl.name, "main");
            assert_eq!(c, "entry");
        } else {
            panic!("expected Statement with label and comment");
        }
    }

    #[test]
    fn parse_indented_instruction_no_label() {
        let result = parse_line("    mov rax, rbx");
        if let Line::Statement {
            label: None,
            body: Some(b),
            comment: None,
        } = result
        {
            assert_eq!(b.mnemonic, "mov");
            assert_eq!(b.operands, vec!["rax", "rbx"]);
        } else {
            panic!("expected instruction without label");
        }
    }

    #[test]
    fn parse_indented_instruction_with_comment() {
        let result = parse_line("    ret ; return");
        if let Line::Statement {
            label: None,
            body: Some(b),
            comment: Some(c),
        } = result
        {
            assert_eq!(b.mnemonic, "ret");
            assert_eq!(c, "return");
        } else {
            panic!("expected instruction with comment");
        }
    }

    #[test]
    fn parse_section_directive() {
        let result = parse_line("section .text");
        if let Line::Statement {
            label: None,
            body: Some(b),
            ..
        } = result
        {
            assert_eq!(b.mnemonic, "section");
            assert_eq!(b.operands, vec![".text"]);
            assert!(b.is_section_level());
        } else {
            panic!("expected section directive");
        }
    }

    #[test]
    fn parse_global_directive() {
        let result = parse_line("global main");
        if let Line::Statement {
            label: None,
            body: Some(b),
            ..
        } = result
        {
            assert_eq!(b.mnemonic, "global");
            assert!(b.is_section_level());
        } else {
            panic!("expected global directive");
        }
    }

    #[test]
    fn parse_bits_directive_uppercased() {
        // BITS should be treated as section-level (after lowercasing)
        let result = parse_line("BITS 64");
        if let Line::Statement {
            label: None,
            body: Some(b),
            ..
        } = result
        {
            assert_eq!(b.mnemonic, "bits");
            assert!(b.is_section_level());
        } else {
            panic!("expected bits directive");
        }
    }

    #[test]
    fn parse_extern_directive() {
        let result = parse_line("extern printf");
        if let Line::Statement {
            label: None,
            body: Some(b),
            ..
        } = result
        {
            assert_eq!(b.mnemonic, "extern");
            assert!(b.is_section_level());
        } else {
            panic!("expected extern directive");
        }
    }

    #[test]
    fn parse_indented_data_definition() {
        // "msg db ..." indented → data label (no colon)
        let result = parse_line("    msg db \"Hello\", 0");
        if let Line::Statement {
            label: Some(lbl),
            body: Some(b),
            ..
        } = result
        {
            assert_eq!(lbl.name, "msg");
            assert!(!lbl.has_colon);
            assert_eq!(b.mnemonic, "db");
        } else {
            panic!("expected data definition");
        }
    }

    #[test]
    fn parse_indented_equ_definition() {
        let result = parse_line("    msg_len equ $ - msg");
        if let Line::Statement {
            label: Some(lbl),
            body: Some(b),
            ..
        } = result
        {
            assert_eq!(lbl.name, "msg_len");
            assert!(!lbl.has_colon);
            assert_eq!(b.mnemonic, "equ");
        } else {
            panic!("expected equ data definition");
        }
    }

    #[test]
    fn parse_indented_resb_definition() {
        let result = parse_line("    buf resb 64");
        if let Line::Statement {
            label: Some(lbl),
            body: Some(b),
            ..
        } = result
        {
            assert_eq!(lbl.name, "buf");
            assert!(!lbl.has_colon);
            assert_eq!(b.mnemonic, "resb");
            assert_eq!(b.operands, vec!["64"]);
        } else {
            panic!("expected resb data definition");
        }
    }

    #[test]
    fn parse_inline_comment_not_in_string() {
        // semicolon inside string: not a comment
        let result = parse_line("    db \"hello; world\"");
        if let Line::Statement {
            comment: None,
            body: Some(b),
            ..
        } = result
        {
            assert_eq!(b.mnemonic, "db");
            assert_eq!(b.operands, vec!["\"hello; world\""]);
        } else {
            panic!("expected db with string containing semicolon, no comment");
        }
    }

    // --- parse (multi-line) ---

    #[test]
    fn parse_empty_source() {
        assert_eq!(parse("").len(), 0);
    }

    #[test]
    fn parse_two_lines() {
        let lines = parse("    mov rax, 0\n    ret\n");
        assert_eq!(lines.len(), 2);
    }

    #[test]
    fn parse_blank_line_in_source() {
        let lines = parse("    ret\n\n    ret\n");
        assert_eq!(lines.len(), 3);
        assert!(matches!(lines[1], Line::Blank));
    }
}
