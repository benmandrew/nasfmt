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
