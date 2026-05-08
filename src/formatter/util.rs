pub(super) const INDENT: usize = 4;
pub(super) const MAX_WIDTH: usize = 80;

pub(super) fn round_up_4(n: usize) -> usize {
    (n + 3) & !3
}

pub(super) fn rtrim(s: &mut String) {
    s.truncate(s.trim_end_matches(' ').len());
}

pub(super) fn spacing(current_col: usize, target_col: usize) -> String {
    if target_col > current_col {
        " ".repeat(target_col - current_col)
    } else {
        "  ".to_string()
    }
}

pub(super) fn wrap_words(text: &str, max_width: usize) -> Vec<String> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_up_4_zero() {
        assert_eq!(round_up_4(0), 0);
    }

    #[test]
    fn round_up_4_one() {
        assert_eq!(round_up_4(1), 4);
    }

    #[test]
    fn round_up_4_three() {
        assert_eq!(round_up_4(3), 4);
    }

    #[test]
    fn round_up_4_exactly_four() {
        assert_eq!(round_up_4(4), 4);
    }

    #[test]
    fn round_up_4_five() {
        assert_eq!(round_up_4(5), 8);
    }

    #[test]
    fn round_up_4_eight() {
        assert_eq!(round_up_4(8), 8);
    }

    #[test]
    fn round_up_4_large() {
        assert_eq!(round_up_4(13), 16);
    }

    #[test]
    fn rtrim_trailing_spaces() {
        let mut s = "hello   ".to_string();
        rtrim(&mut s);
        assert_eq!(s, "hello");
    }

    #[test]
    fn rtrim_no_trailing_spaces() {
        let mut s = "hello".to_string();
        rtrim(&mut s);
        assert_eq!(s, "hello");
    }

    #[test]
    fn rtrim_all_spaces() {
        let mut s = "   ".to_string();
        rtrim(&mut s);
        assert_eq!(s, "");
    }

    #[test]
    fn rtrim_empty() {
        let mut s = String::new();
        rtrim(&mut s);
        assert_eq!(s, "");
    }

    #[test]
    fn rtrim_preserves_interior_spaces() {
        let mut s = "a b c  ".to_string();
        rtrim(&mut s);
        assert_eq!(s, "a b c");
    }

    #[test]
    fn spacing_target_greater() {
        assert_eq!(spacing(5, 10), "     ");
    }

    #[test]
    fn spacing_target_equals_current() {
        assert_eq!(spacing(5, 5), "  ");
    }

    #[test]
    fn spacing_target_less_than_current() {
        assert_eq!(spacing(10, 5), "  ");
    }

    #[test]
    fn spacing_target_zero() {
        assert_eq!(spacing(0, 0), "  ");
    }

    #[test]
    fn spacing_one_apart() {
        assert_eq!(spacing(4, 5), " ");
    }

    #[test]
    fn wrap_words_short_text() {
        assert_eq!(wrap_words("hello", 80), vec!["hello"]);
    }

    #[test]
    fn wrap_words_exact_fit() {
        assert_eq!(wrap_words("hello world", 11), vec!["hello world"]);
    }

    #[test]
    fn wrap_words_one_over() {
        assert_eq!(wrap_words("hello world!", 11), vec!["hello", "world!"]);
    }

    #[test]
    fn wrap_words_multiple_chunks() {
        let result = wrap_words("one two three four five", 9);
        assert_eq!(result, vec!["one two", "three", "four five"]);
    }

    #[test]
    fn wrap_words_single_long_word() {
        let long = "averylongwordthatwontfit";
        let result = wrap_words(long, 10);
        assert_eq!(result, vec![long]);
    }

    #[test]
    fn wrap_words_empty_string() {
        let result = wrap_words("", 80);
        assert_eq!(result, vec![""]);
    }
}
