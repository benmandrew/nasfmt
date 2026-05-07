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

pub(super) fn format_code_label(
    label: &Label,
    body: Option<&Body>,
    comment: Option<&str>,
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

pub(super) fn format_section_directive(body: &Body, comment: Option<&str>) -> String {
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

pub(super) fn format_standalone_comment(indent: usize, text: &str) -> String {
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
