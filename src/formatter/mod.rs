use crate::ast::Line;

mod block;
mod comment;
mod line;
mod operand;
mod util;

pub fn format(lines: &[Line]) -> String {
    let mut output = String::new();
    let mut current_block: Vec<&Line> = vec![];
    for ln in lines {
        match ln {
            Line::Blank => {
                block::flush_block(&mut output, &mut current_block);
                output.push('\n');
            }
            Line::Preprocessor(s) => {
                block::flush_block(&mut output, &mut current_block);
                output.push_str(s);
                output.push('\n');
            }
            Line::CommentOnly { indent, text } => {
                block::flush_block(&mut output, &mut current_block);
                output.push_str(&line::format_standalone_comment(*indent, text));
            }
            Line::Statement {
                label: Some(lbl),
                body,
                comment,
            } if lbl.has_colon => {
                block::flush_block(&mut output, &mut current_block);
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
                block::flush_block(&mut output, &mut current_block);
                output.push_str(&line::format_section_directive(b, comment.as_deref()));
            }
            _ => {
                current_block.push(ln);
            }
        }
    }
    block::flush_block(&mut output, &mut current_block);
    output
}
