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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_text_returns_bare_semicolon() {
        assert_eq!(inline_comment("", 0), ";");
    }

    #[test]
    fn empty_text_any_col_returns_bare_semicolon() {
        assert_eq!(inline_comment("", 40), ";");
    }

    #[test]
    fn short_text_formatted_with_semicolon_space() {
        assert_eq!(inline_comment("hello", 10), "; hello");
    }

    #[test]
    fn text_fits_exactly_at_available_width() {
        // available = 80 - 10 - 2 = 68
        let text = "a".repeat(68);
        assert_eq!(inline_comment(&text, 10), format!("; {}", text));
    }

    #[test]
    fn text_one_char_over_wraps() {
        // available = 80 - 10 - 2 = 68; text of 69 chars must wrap
        let text = "word ".repeat(14).trim().to_string(); // > 68 chars
        let result = inline_comment(&text, 10);
        assert!(result.contains('\n'), "should wrap but got: {:?}", result);
    }

    #[test]
    fn wrapped_continuation_indented_to_comment_col() {
        // comment_col = 20, available = 80 - 20 - 2 = 58
        // Use text that forces a wrap
        let text = "first part that fits and second part that wraps around the column";
        let result = inline_comment(text, 20);
        if result.contains('\n') {
            let second_line = result.split('\n').nth(1).unwrap();
            assert!(second_line.starts_with(&" ".repeat(20)));
            assert!(second_line.contains("; "));
        }
    }

    #[test]
    fn col_near_max_width_available_zero() {
        // comment_col = 78, available = 80 - 78 - 2 = 0 → always fits (no wrap)
        let result = inline_comment("hi", 78);
        assert_eq!(result, "; hi");
    }
}
