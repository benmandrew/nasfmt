use crate::ast::{Body, Label, Line};

const INDENT: usize = 4;
const MAX_WIDTH: usize = 80;

const REGISTERS: &[&str] = &[
    "rax", "rbx", "rcx", "rdx", "rsi", "rdi", "rsp", "rbp", "r8", "r9", "r10", "r11", "r12", "r13",
    "r14", "r15", "eax", "ebx", "ecx", "edx", "esi", "edi", "esp", "ebp", "r8d", "r9d", "r10d",
    "r11d", "r12d", "r13d", "r14d", "r15d", "ax", "bx", "cx", "dx", "si", "di", "sp", "bp", "r8w",
    "r9w", "r10w", "r11w", "r12w", "r13w", "r14w", "r15w", "al", "bl", "cl", "dl", "sil", "dil",
    "spl", "bpl", "ah", "bh", "ch", "dh", "r8b", "r9b", "r10b", "r11b", "r12b", "r13b", "r14b",
    "r15b", "cs", "ds", "es", "fs", "gs", "ss", "cr0", "cr2", "cr3", "cr4", "cr8", "dr0", "dr1",
    "dr2", "dr3", "dr6", "dr7", "mm0", "mm1", "mm2", "mm3", "mm4", "mm5", "mm6", "mm7", "xmm0",
    "xmm1", "xmm2", "xmm3", "xmm4", "xmm5", "xmm6", "xmm7", "xmm8", "xmm9", "xmm10", "xmm11",
    "xmm12", "xmm13", "xmm14", "xmm15", "ymm0", "ymm1", "ymm2", "ymm3", "ymm4", "ymm5", "ymm6",
    "ymm7", "ymm8", "ymm9", "ymm10", "ymm11", "ymm12", "ymm13", "ymm14", "ymm15", "rip", "eip",
    "st0", "st1", "st2", "st3", "st4", "st5", "st6", "st7", "rel", "abs", "nosplit",
];

const SIZE_PREFIXES: &[&str] = &[
    "byte", "word", "dword", "qword", "tword", "oword", "yword", "zword", "short", "near", "far",
];

pub fn format(lines: &[Line]) -> String {
    let mut output = String::new();
    let mut block: Vec<&Line> = vec![];

    for line in lines {
        match line {
            Line::Blank => {
                flush_block(&mut output, &mut block);
                output.push('\n');
            }
            Line::Preprocessor(s) => {
                flush_block(&mut output, &mut block);
                output.push_str(s);
                output.push('\n');
            }
            Line::CommentOnly { indent, text } => {
                flush_block(&mut output, &mut block);
                output.push_str(&format_standalone_comment(*indent, text));
            }
            Line::Statement {
                label: Some(lbl),
                body,
                comment,
            } if lbl.has_colon => {
                flush_block(&mut output, &mut block);
                output.push_str(&format_code_label(lbl, body.as_ref(), comment.as_deref()));
            }
            Line::Statement {
                label: None,
                body: Some(b),
                comment,
            } if b.is_section_level() => {
                flush_block(&mut output, &mut block);
                output.push_str(&format_section_directive(b, comment.as_deref()));
            }
            _ => {
                block.push(line);
            }
        }
    }

    flush_block(&mut output, &mut block);
    output
}

fn flush_block(output: &mut String, block: &mut Vec<&Line>) {
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

// ── Block formatters ──────────────────────────────────────────────────────────

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

    let indent = " ".repeat(INDENT);
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
                let label_field = format!("{:<width$}", name, width = label_width);
                match body {
                    Some(b) => {
                        let mnemonic_field =
                            format!("{:<width$}", b.mnemonic, width = mnemonic_width);
                        let ops = fmt_ops[i].as_deref().unwrap_or("");
                        let content = format!("{}{}{}", label_field, mnemonic_field, ops);
                        let content_col = INDENT + content.len();
                        out.push_str(&indent);
                        out.push_str(&content);
                        if let Some(c) = comment {
                            let spaces = spacing(content_col, comment_col);
                            out.push_str(&spaces);
                            out.push_str(&inline_comment(c, comment_col));
                        }
                        out.push('\n');
                    }
                    None => {
                        out.push_str(&indent);
                        out.push_str(name);
                        if let Some(c) = comment {
                            let col = INDENT + name.len();
                            let spaces = spacing(col, INDENT + label_width + 4);
                            let cc = INDENT + label_width + 4;
                            out.push_str(&spaces);
                            out.push_str(&inline_comment(c, cc));
                        }
                        out.push('\n');
                    }
                }
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

// ── Individual line formatters ────────────────────────────────────────────────

fn format_instr_line(
    mnemonic: &str,
    mnemonic_width: usize,
    operands: Option<&str>,
    comment: Option<&str>,
    comment_col: usize,
) -> String {
    let indent = " ".repeat(INDENT);
    let content = match operands {
        Some(ops) if !ops.is_empty() => {
            format!("{:<width$}{}", mnemonic, ops, width = mnemonic_width)
        }
        _ => mnemonic.to_string(),
    };
    let content_col = INDENT + content.len();

    match comment {
        None => format!("{}{}\n", indent, content),
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

fn format_code_label(label: &Label, body: Option<&Body>, comment: Option<&str>) -> String {
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
                Some(format_operands(&b.operands))
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
            ));
        }
    }
    out
}

fn format_section_directive(body: &Body, comment: Option<&str>) -> String {
    let content = if body.operands.is_empty() {
        body.mnemonic.clone()
    } else {
        format!("{} {}", body.mnemonic, format_operands(&body.operands))
    };
    match comment {
        None => format!("{}\n", content),
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

fn format_standalone_comment(indent: usize, text: &str) -> String {
    let prefix = " ".repeat(indent);
    if text.is_empty() {
        return format!("{};\n", prefix);
    }
    let available = MAX_WIDTH.saturating_sub(indent + 2);
    if text.len() <= available {
        return format!("{}; {}\n", prefix, text);
    }
    let chunks = wrap_words(text, available);
    chunks
        .into_iter()
        .map(|chunk| format!("{}; {}\n", prefix, chunk))
        .collect()
}

// ── Operand formatting ────────────────────────────────────────────────────────

fn format_operands(operands: &[String]) -> String {
    operands
        .iter()
        .map(|op| format_operand(op))
        .collect::<Vec<_>>()
        .join(", ")
}

fn format_operand(s: &str) -> String {
    let s = s.trim();
    let s_lower = s.to_lowercase();

    for prefix in SIZE_PREFIXES {
        if s_lower.starts_with(prefix) {
            let rest = s[prefix.len()..].trim_start();
            return format!("{} {}", prefix, format_operand_inner(rest));
        }
    }
    format_operand_inner(s)
}

fn format_operand_inner(s: &str) -> String {
    let s = s.trim();
    if s.starts_with('[') && s.ends_with(']') {
        return format!("[{}]", format_mem_expr(&s[1..s.len() - 1]));
    }
    let has_operator = s.chars().any(|c| matches!(c, '+' | '-' | '*' | '/'));
    if has_operator {
        format_mem_expr(s)
    } else {
        format_token(s)
    }
}

fn format_mem_expr(s: &str) -> String {
    let mut result = String::new();
    let mut chars = s.char_indices().peekable();

    while let Some((i, c)) = chars.next() {
        match c {
            '"' | '\'' | '`' => {
                let q = c;
                result.push(c);
                while let Some((_, nc)) = chars.next() {
                    result.push(nc);
                    if nc == q {
                        break;
                    }
                    if nc == '\\' && q != '\''
                        && let Some((_, ec)) = chars.next() {
                            result.push(ec);
                        }
                }
            }
            ' ' | '\t' => {}
            '+' | '*' | '/' => {
                rtrim(&mut result);
                result.push(' ');
                result.push(c);
                result.push(' ');
            }
            '-' => {
                let last = result.trim_end().chars().last();
                let is_binary = matches!(
                    last,
                    Some(lc) if lc.is_alphanumeric() || matches!(lc, '_' | '$' | ')' | ']' | '\'' | '"')
                );
                rtrim(&mut result);
                if is_binary {
                    result.push_str(" - ");
                } else {
                    result.push('-');
                }
            }
            _ if c.is_alphanumeric() || matches!(c, '_' | '$' | '.' | '@') => {
                let start = i;
                let mut end = i + c.len_utf8();
                while let Some(&(j, nc)) = chars.peek() {
                    if nc.is_alphanumeric() || matches!(nc, '_' | '$' | '.' | '@') {
                        end = j + nc.len_utf8();
                        chars.next();
                    } else {
                        break;
                    }
                }
                result.push_str(&format_token(&s[start..end]));
            }
            _ => {
                result.push(c);
            }
        }
    }

    result.trim_end().to_string()
}

fn format_token(s: &str) -> String {
    if !s.bytes().any(|b| b.is_ascii_uppercase()) {
        return s.to_string();
    }
    let lower = s.to_lowercase();
    if REGISTERS.contains(&lower.as_str()) || SIZE_PREFIXES.contains(&lower.as_str()) {
        lower
    } else {
        s.to_string()
    }
}

// ── Comment helpers ───────────────────────────────────────────────────────────

fn inline_comment(text: &str, comment_col: usize) -> String {
    if text.is_empty() {
        return ";".to_string();
    }
    let available = MAX_WIDTH.saturating_sub(comment_col + 2);
    if available == 0 || text.len() <= available {
        return format!("; {}", text);
    }
    let chunks = wrap_words(text, available);
    let cont = format!("\n{}; ", " ".repeat(comment_col));
    format!("; {}", chunks.join(&cont))
}

fn wrap_words(text: &str, max_width: usize) -> Vec<String> {
    if text.len() <= max_width {
        return vec![text.to_string()];
    }
    let mut lines: Vec<String> = vec![];
    let mut current = String::new();
    for word in text.split_whitespace() {
        if current.is_empty() {
            current = word.to_string();
        } else if current.len() + 1 + word.len() <= max_width {
            current.push(' ');
            current.push_str(word);
        } else {
            lines.push(current);
            current = word.to_string();
        }
    }
    if !current.is_empty() {
        lines.push(current);
    }
    lines
}

// ── Utilities ─────────────────────────────────────────────────────────────────

fn round_up_4(n: usize) -> usize {
    (n + 3) & !3
}

fn rtrim(s: &mut String) {
    s.truncate(s.trim_end_matches(' ').len());
}

fn spacing(current_col: usize, target_col: usize) -> String {
    if target_col > current_col {
        " ".repeat(target_col - current_col)
    } else {
        "  ".to_string()
    }
}
