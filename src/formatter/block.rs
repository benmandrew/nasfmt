use super::comment::inline_comment;
use super::line::format_instr_line;
use super::operand::format_operands;
use super::util::{INDENT, round_up_4, spacing};
use crate::ast::{Label, Line};

struct BlockMetrics {
    mnemonic_width: usize,
    fmt_ops: Vec<Option<String>>,
    comment_col: usize,
}

fn compute_block_metrics(block: &[&Line], label_prefix_width: usize) -> BlockMetrics {
    let max_mnemonic = block
        .iter()
        .filter_map(|l| match l {
            Line::Statement { body: Some(b), .. } => Some(b.mnemonic.len()),
            _ => None,
        })
        .max()
        .unwrap_or(0);
    let mnemonic_width = round_up_4(max_mnemonic + 1);
    let fmt_ops: Vec<Option<String>> = block
        .iter()
        .map(|l| match l {
            Line::Statement { body: Some(b), .. } if !b.operands.is_empty() => {
                Some(format_operands(&b.operands))
            }
            _ => None,
        })
        .collect();

    let max_ops = fmt_ops
        .iter()
        .filter_map(|o| o.as_ref().map(|s| s.len()))
        .max()
        .unwrap_or(0);
    let has_comments = block.iter().any(|l| {
        matches!(
            l,
            Line::Statement {
                comment: Some(_),
                ..
            }
        )
    });
    let comment_col = if has_comments {
        INDENT + label_prefix_width + mnemonic_width + max_ops + 4
    } else {
        0
    };
    BlockMetrics {
        mnemonic_width,
        fmt_ops,
        comment_col,
    }
}

pub(super) fn flush_block(output: &mut String, block: &mut Vec<&Line>) {
    if block.is_empty() {
        return;
    }
    let has_data_labels = block.iter().any(|l| {
        matches!(
            l,
            Line::Statement {
                label: Some(Label {
                    has_colon: false,
                    ..
                }),
                ..
            }
        )
    });
    if has_data_labels {
        output.push_str(&format_data_block(block));
    } else {
        output.push_str(&format_instruction_block(block));
    }
    block.clear();
}

fn format_instruction_block(block: &[&Line]) -> String {
    let BlockMetrics {
        mnemonic_width,
        fmt_ops,
        comment_col,
    } = compute_block_metrics(block, 0);
    let mut out = String::new();
    for (i, line) in block.iter().enumerate() {
        match line {
            Line::Statement {
                body: Some(b),
                comment,
                ..
            } => {
                out.push_str(&format_instr_line(
                    &b.mnemonic,
                    mnemonic_width,
                    fmt_ops[i].as_deref(),
                    comment.as_deref(),
                    comment_col,
                ));
            }
            _ => out.push('\n'),
        }
    }
    out
}

fn format_data_block(block: &[&Line]) -> String {
    let max_label = block
        .iter()
        .filter_map(|l| match l {
            Line::Statement {
                label:
                    Some(Label {
                        name,
                        has_colon: false,
                    }),
                ..
            } => Some(name.len()),
            _ => None,
        })
        .max()
        .unwrap_or(0);
    let label_width = round_up_4(max_label + 1);
    let BlockMetrics {
        mnemonic_width,
        fmt_ops,
        comment_col,
    } = compute_block_metrics(block, label_width);
    let mut out = String::new();
    for (i, line) in block.iter().enumerate() {
        match line {
            Line::Statement {
                label:
                    Some(Label {
                        name,
                        has_colon: false,
                    }),
                body,
                comment,
            } => {
                out.push_str(&format_data_label_line(
                    name,
                    label_width,
                    mnemonic_width,
                    body.as_ref().map(|b| b.mnemonic.as_str()),
                    fmt_ops[i].as_deref(),
                    comment.as_deref(),
                    comment_col,
                ));
            }
            Line::Statement {
                label: None,
                body: Some(b),
                comment,
                ..
            } => {
                out.push_str(&format_instr_line(
                    &b.mnemonic,
                    mnemonic_width,
                    fmt_ops[i].as_deref(),
                    comment.as_deref(),
                    comment_col,
                ));
            }
            _ => out.push('\n'),
        }
    }
    out
}

fn format_data_label_line(
    name: &str,
    label_width: usize,
    mnemonic_width: usize,
    mnemonic: Option<&str>,
    ops: Option<&str>,
    comment: Option<&str>,
    comment_col: usize,
) -> String {
    let indent = " ".repeat(INDENT);
    let mut out = indent;
    match mnemonic {
        Some(m) => {
            let label_field = format!("{:<width$}", name, width = label_width);
            let mnemonic_field = format!("{:<width$}", m, width = mnemonic_width);
            let content = format!("{}{}{}", label_field, mnemonic_field, ops.unwrap_or(""));
            let content_col = INDENT + content.len();
            out.push_str(&content);
            if let Some(c) = comment {
                out.push_str(&spacing(content_col, comment_col));
                out.push_str(&inline_comment(c, comment_col));
            }
        }
        None => {
            out.push_str(name);
            if let Some(c) = comment {
                let col = INDENT + name.len();
                let cc = INDENT + label_width + 4;
                out.push_str(&spacing(col, cc));
                out.push_str(&inline_comment(c, cc));
            }
        }
    }
    out.push('\n');
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Body, Label, Line};

    fn instr(mnemonic: &str, operands: &[&str]) -> Line {
        Line::Statement {
            label: None,
            body: Some(Body {
                mnemonic: mnemonic.to_string(),
                operands: operands.iter().map(|s| s.to_string()).collect(),
            }),
            comment: None,
        }
    }

    fn instr_comment(mnemonic: &str, operands: &[&str], comment: &str) -> Line {
        Line::Statement {
            label: None,
            body: Some(Body {
                mnemonic: mnemonic.to_string(),
                operands: operands.iter().map(|s| s.to_string()).collect(),
            }),
            comment: Some(comment.to_string()),
        }
    }

    fn data(name: &str, mnemonic: &str, operands: &[&str]) -> Line {
        Line::Statement {
            label: Some(Label {
                name: name.to_string(),
                has_colon: false,
            }),
            body: Some(Body {
                mnemonic: mnemonic.to_string(),
                operands: operands.iter().map(|s| s.to_string()).collect(),
            }),
            comment: None,
        }
    }

    fn data_comment(name: &str, mnemonic: &str, operands: &[&str], comment: &str) -> Line {
        Line::Statement {
            label: Some(Label {
                name: name.to_string(),
                has_colon: false,
            }),
            body: Some(Body {
                mnemonic: mnemonic.to_string(),
                operands: operands.iter().map(|s| s.to_string()).collect(),
            }),
            comment: Some(comment.to_string()),
        }
    }

    // --- flush_block ---

    #[test]
    fn flush_empty_block_no_output() {
        let mut out = String::new();
        let mut block: Vec<&Line> = vec![];
        flush_block(&mut out, &mut block);
        assert_eq!(out, "");
        assert!(block.is_empty());
    }

    #[test]
    fn flush_clears_block() {
        let l = instr("ret", &[]);
        let mut out = String::new();
        let mut block = vec![&l];
        flush_block(&mut out, &mut block);
        assert!(block.is_empty());
    }

    #[test]
    fn flush_single_instruction() {
        let l = instr("ret", &[]);
        let mut out = String::new();
        let mut block = vec![&l];
        flush_block(&mut out, &mut block);
        assert_eq!(out, "    ret\n");
    }

    #[test]
    fn flush_instruction_with_operands() {
        let l = instr("mov", &["rax", "0"]);
        let mut out = String::new();
        let mut block = vec![&l];
        flush_block(&mut out, &mut block);
        assert_eq!(out, "    mov rax, 0\n");
    }

    #[test]
    fn flush_aligns_mnemonics_in_block() {
        // push (4 chars) is longest; mnemonic_width = round_up_4(5) = 8
        let push_l = instr("push", &["rbx"]);
        let mov_l = instr("mov", &["rax", "0"]);
        let ret_l = instr("ret", &[]);
        let mut out = String::new();
        let mut block = vec![&push_l, &mov_l, &ret_l];
        flush_block(&mut out, &mut block);
        assert_eq!(out, "    push    rbx\n    mov     rax, 0\n    ret\n");
    }

    #[test]
    fn flush_aligns_comments_in_block() {
        // "ret" and "xor eax, eax" — both len-3 mnemonics, mnemonic_width=4
        // max_ops = len("eax, eax") = 8, comment_col = 4+0+4+8+4 = 20
        let ret_l = instr_comment("ret", &[], "end");
        let xor_l = instr_comment("xor", &["eax", "eax"], "zero");
        let mut out = String::new();
        let mut block = vec![&ret_l, &xor_l];
        flush_block(&mut out, &mut block);
        // Both comments must start at column 20
        for line in out.lines() {
            if line.contains(';') {
                let semicolon_col = line.find(';').unwrap();
                assert_eq!(semicolon_col, 20, "comment not at col 20 in: {:?}", line);
            }
        }
    }

    #[test]
    fn flush_data_block_aligns_labels() {
        // msg (3), newline (7) → label_width = round_up_4(8) = 8
        let msg_l = data("msg", "db", &["0"]);
        let newline_l = data("newline", "db", &["10"]);
        let mut out = String::new();
        let mut block = vec![&msg_l, &newline_l];
        flush_block(&mut out, &mut block);
        assert!(out.contains("    msg     "), "msg not padded: {:?}", out);
        assert!(
            out.contains("    newline "),
            "newline not padded: {:?}",
            out
        );
    }

    #[test]
    fn flush_data_block_aligns_mnemonics() {
        let msg_l = data("msg", "db", &["\"hi\""]);
        let len_l = data("msg_len", "equ", &["$ - msg"]);
        let mut out = String::new();
        let mut block = vec![&msg_l, &len_l];
        flush_block(&mut out, &mut block);
        // "db" and "equ" → max=3, mnemonic_width = round_up_4(4) = 4
        // both directives padded to 4
        assert!(out.contains("db  "), "db not padded: {:?}", out);
        assert!(out.contains("equ "), "equ not padded: {:?}", out);
    }

    #[test]
    fn flush_data_block_with_comment() {
        let l = data_comment("x", "db", &["1"], "value");
        let mut out = String::new();
        let mut block = vec![&l];
        flush_block(&mut out, &mut block);
        assert!(out.contains("; value"), "missing comment: {:?}", out);
    }
}
