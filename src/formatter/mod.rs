use crate::ast::Line;

mod block;
mod comment;
mod line;
mod operand;
mod util;

use operand::format_operands;
use util::{INDENT, round_up_4};

struct FileMetrics {
    /// Mnemonic column width for all instruction (code) blocks in the file.
    pub code_mnemonic_width: usize,
    /// Comment column for all instruction (code) blocks in the file.
    /// 0 means no inline comments anywhere in code blocks.
    pub code_comment_col: usize,
}

fn is_code_block_line(ln: &Line) -> bool {
    match ln {
        Line::Blank | Line::Preprocessor(_) | Line::CommentOnly { .. } => false,
        Line::Statement {
            label: Some(lbl), ..
        } if lbl.has_colon => false,
        // Data labels (no colon) are not code lines
        Line::Statement { label: Some(_), .. } => false,
        Line::Statement {
            label: None,
            body: Some(b),
            ..
        } if b.is_section_level() => false,
        _ => true,
    }
}

fn compute_file_metrics(lines: &[Line]) -> FileMetrics {
    let max_mnemonic = lines
        .iter()
        .filter(|l| is_code_block_line(l))
        .filter_map(|l| match l {
            Line::Statement { body: Some(b), .. } => Some(b.mnemonic.len()),
            _ => None,
        })
        .max()
        .unwrap_or(0);
    let code_mnemonic_width = round_up_4(max_mnemonic + 1);

    let has_comments = lines.iter().filter(|l| is_code_block_line(l)).any(|l| {
        matches!(
            l,
            Line::Statement {
                comment: Some(_),
                ..
            }
        )
    });

    // Comment column is derived only from lines that actually carry comments,
    // so that uncommented lines with very long operands don't inflate the column.
    let code_comment_col = if has_comments {
        let max_content = lines
            .iter()
            .filter(|l| is_code_block_line(l))
            .filter_map(|l| match l {
                Line::Statement {
                    body: Some(b),
                    comment: Some(_),
                    ..
                } => {
                    // When there are no operands, the mnemonic is NOT padded.
                    let content = if b.operands.is_empty() {
                        INDENT + b.mnemonic.len()
                    } else {
                        INDENT + code_mnemonic_width + format_operands(&b.operands).len()
                    };
                    Some(content)
                }
                _ => None,
            })
            .max()
            .unwrap_or(0);
        max_content + 4
    } else {
        0
    };

    FileMetrics {
        code_mnemonic_width,
        code_comment_col,
    }
}

/// Returns true when the CommentOnly at `pos` is a wrapped continuation of an
/// inline comment on a preceding Statement. Continuations have a large indent
/// (> INDENT, matching the comment column) and no blank line separating them
/// from the Statement. Multi-line chains (`stmt ; a` / `; b` / `; c`) are also
/// recognised.
fn is_comment_continuation(lines: &[Line], pos: usize, indent: usize) -> bool {
    if indent <= INDENT {
        return false;
    }
    for ln in lines[..pos].iter().rev() {
        match ln {
            Line::Blank => return false,
            Line::Statement {
                comment: Some(_), ..
            } => return true,
            Line::CommentOnly { .. } => {}
            _ => return false,
        }
    }
    false
}

/// Returns the indent that a standalone comment at `pos` should use, based on
/// the next non-blank, non-comment line. Returns `None` when there is no such
/// line (end of file), so the caller can fall back to the original indent.
fn comment_indent_for(lines: &[Line], pos: usize) -> Option<usize> {
    for ln in lines[pos + 1..].iter() {
        match ln {
            Line::Blank | Line::CommentOnly { .. } => continue,
            Line::Preprocessor(_) => return Some(0),
            Line::Statement {
                label: Some(lbl), ..
            } if lbl.has_colon => return Some(0),
            Line::Statement {
                label: None,
                body: Some(b),
                ..
            } if b.is_section_level() => return Some(0),
            _ => return Some(INDENT),
        }
    }
    None
}

pub fn format(lines: &[Line]) -> String {
    let FileMetrics {
        code_mnemonic_width,
        code_comment_col,
    } = compute_file_metrics(lines);
    let mut output = String::new();
    let mut current_block: Vec<&Line> = vec![];
    for (i, ln) in lines.iter().enumerate() {
        match ln {
            Line::Blank => {
                block::flush_block(
                    &mut output,
                    &mut current_block,
                    code_mnemonic_width,
                    code_comment_col,
                );
                output.push('\n');
            }
            Line::Preprocessor(s) => {
                block::flush_block(
                    &mut output,
                    &mut current_block,
                    code_mnemonic_width,
                    code_comment_col,
                );
                output.push_str(s);
                output.push('\n');
            }
            Line::CommentOnly { indent, text } => {
                block::flush_block(
                    &mut output,
                    &mut current_block,
                    code_mnemonic_width,
                    code_comment_col,
                );
                let effective_indent = if is_comment_continuation(lines, i, *indent) {
                    *indent
                } else {
                    comment_indent_for(lines, i).unwrap_or(*indent)
                };
                output.push_str(&line::format_standalone_comment(effective_indent, text));
            }
            Line::Statement {
                label: Some(lbl),
                body,
                comment,
            } if lbl.has_colon => {
                block::flush_block(
                    &mut output,
                    &mut current_block,
                    code_mnemonic_width,
                    code_comment_col,
                );
                output.push_str(&line::format_code_label(
                    lbl,
                    body.as_ref(),
                    comment.as_deref(),
                ));
            }
            Line::Statement {
                label: None,
                body: Some(b),
                comment,
            } if b.is_section_level() => {
                block::flush_block(
                    &mut output,
                    &mut current_block,
                    code_mnemonic_width,
                    code_comment_col,
                );
                output.push_str(&line::format_section_directive(b, comment.as_deref()));
            }
            _ => {
                current_block.push(ln);
            }
        }
    }
    block::flush_block(
        &mut output,
        &mut current_block,
        code_mnemonic_width,
        code_comment_col,
    );
    output
}
