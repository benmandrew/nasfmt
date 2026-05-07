use super::util::{MAX_WIDTH, wrap_words};

pub(super) fn inline_comment(text: &str, comment_col: usize) -> String {
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
