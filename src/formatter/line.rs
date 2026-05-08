use super::comment::inline_comment;
use super::operand::format_operands;
use super::util::{INDENT, MAX_WIDTH, round_up_4, spacing, wrap_words};
use crate::ast::{Body, Label};

pub(super) fn format_instr_line(
    mnemonic: &str,
    mnemonic_width: usize,
    operands: Option<&str>,
    comment: Option<&str>,
    comment_col: usize,
    upper: bool,
) -> String {
    let indent = " ".repeat(INDENT);
    let mnemonic_cased = if upper {
        mnemonic.to_uppercase()
    } else {
        mnemonic.to_string()
    };
    let content = match operands {
        Some(ops) if !ops.is_empty() => {
            format!("{mnemonic_cased:<mnemonic_width$}{ops}")
        }
        _ => mnemonic_cased,
    };
    let content_col = INDENT + content.len();
    match comment {
        None => format!("{indent}{content}\n"),
        Some(c) => {
            let spaces = spacing(content_col, comment_col);
            format!(
                "{}{}{}{}\n",
                indent,
                content,
                spaces,
                inline_comment(c, comment_col)
            )
        }
    }
}

pub(super) fn format_code_label(
    label: &Label,
    body: Option<&Body>,
    comment: Option<&str>,
    upper: bool,
) -> String {
    let label_str = format!("{}:", label.name);
    let mut out = String::new();
    match (body, comment) {
        (None, None) => {
            out.push_str(&label_str);
            out.push('\n');
        }
        (None, Some(c)) => {
            let cc = label_str.len() + 4;
            out.push_str(&label_str);
            out.push_str(&spacing(label_str.len(), cc));
            out.push_str(&inline_comment(c, cc));
            out.push('\n');
        }
        (Some(b), comment) => {
            out.push_str(&label_str);
            out.push('\n');
            let ops = if b.operands.is_empty() {
                None
            } else {
                Some(format_operands(&b.operands, upper))
            };
            let mw = round_up_4(b.mnemonic.len() + 1);
            let cc = if comment.is_some() {
                INDENT + mw + ops.as_deref().map_or(0, |s| s.len()) + 4
            } else {
                0
            };
            out.push_str(&format_instr_line(
                &b.mnemonic,
                mw,
                ops.as_deref(),
                comment,
                cc,
                upper,
            ));
        }
    }
    out
}

pub(super) fn format_section_directive(body: &Body, comment: Option<&str>, upper: bool) -> String {
    let mnemonic_cased = if upper {
        body.mnemonic.to_uppercase()
    } else {
        body.mnemonic.clone()
    };
    let content = if body.operands.is_empty() {
        mnemonic_cased
    } else {
        format!(
            "{} {}",
            mnemonic_cased,
            format_operands(&body.operands, upper)
        )
    };
    match comment {
        None => format!("{content}\n"),
        Some(c) => {
            let cc = content.len() + 4;
            format!(
                "{}{}{}\n",
                content,
                spacing(content.len(), cc),
                inline_comment(c, cc)
            )
        }
    }
}

pub(super) fn format_standalone_comment(indent: usize, text: &str) -> String {
    let prefix = " ".repeat(indent);
    if text.is_empty() {
        return format!("{prefix};\n");
    }
    let available = MAX_WIDTH.saturating_sub(indent + 2);
    if text.len() <= available {
        return format!("{prefix}; {text}\n");
    }
    let chunks = wrap_words(text, available);
    chunks
        .into_iter()
        .map(|chunk| format!("{prefix}; {chunk}\n"))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Body, Label};

    fn body(mnemonic: &str, operands: &[&str]) -> Body {
        Body {
            mnemonic: mnemonic.to_string(),
            operands: operands.iter().map(|s| s.to_string()).collect(),
        }
    }

    fn label(name: &str) -> Label {
        Label {
            name: name.to_string(),
            has_colon: true,
        }
    }

    // --- format_instr_line ---

    #[test]
    fn instr_no_ops_no_comment() {
        assert_eq!(
            format_instr_line("ret", 8, None, None, 0, false),
            "    ret\n"
        );
    }

    #[test]
    fn instr_with_ops() {
        assert_eq!(
            format_instr_line("mov", 8, Some("rax, rbx"), None, 0, false),
            "    mov     rax, rbx\n"
        );
    }

    #[test]
    fn instr_mnemonic_padded_to_width() {
        assert_eq!(
            format_instr_line("mov", 4, Some("rax"), None, 0, false),
            "    mov rax\n"
        );
    }

    #[test]
    fn instr_with_comment_at_col() {
        let result = format_instr_line("ret", 4, None, Some("done"), 12, false);
        assert_eq!(result, "    ret     ; done\n");
    }

    #[test]
    fn instr_empty_ops_treated_as_no_ops() {
        assert_eq!(
            format_instr_line("ret", 4, Some(""), None, 0, false),
            "    ret\n"
        );
    }

    #[test]
    fn instr_mnemonic_uppercased() {
        assert_eq!(
            format_instr_line("mov", 8, Some("RAX, RBX"), None, 0, true),
            "    MOV     RAX, RBX\n"
        );
    }

    // --- format_code_label ---

    #[test]
    fn code_label_bare() {
        let lbl = label("main");
        assert_eq!(format_code_label(&lbl, None, None, false), "main:\n");
    }

    #[test]
    fn code_label_with_comment_no_body() {
        let lbl = label("main");
        let result = format_code_label(&lbl, None, Some("entry point"), false);
        assert_eq!(result, "main:    ; entry point\n");
    }

    #[test]
    fn code_label_with_body_no_comment() {
        let lbl = label("init");
        let b = body("mov", &["rax", "0"]);
        let result = format_code_label(&lbl, Some(&b), None, false);
        assert_eq!(result, "init:\n    mov rax, 0\n");
    }

    #[test]
    fn code_label_with_body_and_comment() {
        let lbl = label("start");
        let b = body("xor", &["eax", "eax"]);
        let result = format_code_label(&lbl, Some(&b), Some("zero eax"), false);
        assert!(result.starts_with("start:\n"));
        assert!(result.contains("; zero eax"));
    }

    #[test]
    fn code_label_body_no_operands() {
        let lbl = label("end");
        let b = body("ret", &[]);
        let result = format_code_label(&lbl, Some(&b), None, false);
        assert_eq!(result, "end:\n    ret\n");
    }

    #[test]
    fn code_label_with_body_uppercased() {
        let lbl = label("init");
        let b = body("mov", &["rax", "0"]);
        let result = format_code_label(&lbl, Some(&b), None, true);
        assert_eq!(result, "init:\n    MOV RAX, 0\n");
    }

    // --- format_section_directive ---

    #[test]
    fn section_directive_no_operands() {
        let b = body("bits", &[]);
        assert_eq!(format_section_directive(&b, None, false), "bits\n");
    }

    #[test]
    fn section_directive_with_operand() {
        let b = body("section", &[".text"]);
        assert_eq!(format_section_directive(&b, None, false), "section .text\n");
    }

    #[test]
    fn section_directive_global() {
        let b = body("global", &["main"]);
        assert_eq!(format_section_directive(&b, None, false), "global main\n");
    }

    #[test]
    fn section_directive_with_comment() {
        let b = body("global", &["_start"]);
        let result = format_section_directive(&b, Some("entry point"), false);
        assert_eq!(result, "global _start    ; entry point\n");
    }

    #[test]
    fn section_directive_extern_multiple_operands() {
        let b = body("extern", &["foo", "bar"]);
        assert_eq!(
            format_section_directive(&b, None, false),
            "extern foo, bar\n"
        );
    }

    #[test]
    fn section_directive_uppercased() {
        let b = body("section", &[".text"]);
        assert_eq!(format_section_directive(&b, None, true), "SECTION .text\n");
    }

    // --- format_standalone_comment ---

    #[test]
    fn standalone_comment_empty_text() {
        assert_eq!(format_standalone_comment(0, ""), ";\n");
    }

    #[test]
    fn standalone_comment_simple() {
        assert_eq!(format_standalone_comment(0, "hello"), "; hello\n");
    }

    #[test]
    fn standalone_comment_with_indent() {
        assert_eq!(format_standalone_comment(4, "hello"), "    ; hello\n");
    }

    #[test]
    fn standalone_comment_wraps_long_text() {
        // indent=0, available = 80 - 2 = 78. 20 "word " = 100 chars → must wrap
        let words: Vec<&str> = (0..20).map(|_| "word").collect();
        let text = words.join(" ");
        let result = format_standalone_comment(0, &text);
        assert!(result.contains('\n'));
        for line in result.lines() {
            assert!(line.len() <= 80, "line too long: {line:?}");
        }
    }

    #[test]
    fn standalone_comment_each_wrapped_line_has_prefix() {
        let text = "a b c d e f g h i j k l m n o p q r s t u v w x y z aa bb cc dd ee ff gg hh";
        let result = format_standalone_comment(4, text);
        for line in result.lines() {
            assert!(line.starts_with("    ; "));
        }
    }
}
