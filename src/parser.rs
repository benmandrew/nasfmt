use crate::ast::{Body, Label, Line, SECTION_DIRECTIVES};

const DATA_DIRECTIVES: &[&str] = &[
    "db", "dw", "dd", "dq", "dt", "ddq", "do", "resb", "resw", "resd", "resq", "rest", "resdq",
    "equ", "times", "incbin",
];

pub fn parse(source: &str) -> Vec<Line> {
    source.lines().map(parse_line).collect()
}

fn find_comment_start(s: &str) -> Option<usize> {
    let mut chars = s.char_indices().peekable();
    while let Some((i, c)) = chars.next() {
        match c {
            '"' | '\'' | '`' => {
                let q = c;
                while let Some((_, nc)) = chars.next() {
                    if nc == q {
                        break;
                    }
                    if nc == '\\' && q != '\'' {
                        chars.next();
                    }
                }
            }
            ';' => return Some(i),
            _ => {}
        }
    }
    None
}

fn parse_line(line: &str) -> Line {
    let trimmed_end = line.trim_end();
    if trimmed_end.is_empty() {
        return Line::Blank;
    }

    let trimmed = trimmed_end.trim_start();
    if trimmed.starts_with('%') {
        return Line::Preprocessor(trimmed_end.to_string());
    }

    let (code_raw, comment) = match find_comment_start(trimmed_end) {
        Some(pos) => {
            let raw = &trimmed_end[pos + 1..];
            let text = raw.strip_prefix(' ').unwrap_or(raw).to_string();
            (&trimmed_end[..pos], Some(text))
        }
        None => (trimmed_end, None),
    };

    let code = code_raw.trim_end();

    if code.trim().is_empty() {
        let indent = code_raw.len();
        return match comment {
            Some(text) => Line::CommentOnly { indent, text },
            None => Line::Blank, // bare `;` with no text handled via empty CommentOnly below
        };
    }

    let starts_indented = line.starts_with(|c: char| c.is_whitespace());
    parse_statement(code.trim(), starts_indented, comment)
}

fn parse_statement(code: &str, starts_indented: bool, comment: Option<String>) -> Line {
    let (first, rest) = split_first_token(code);
    let first_lower = first.to_lowercase();

    if starts_indented {
        // Indented data definitions: `    label db ...` / `    label equ ...`
        // Detect by checking if second token is a known data directive.
        let rest_trimmed = rest.trim_start();
        let (second, _) = split_first_token(rest_trimmed);
        if DATA_DIRECTIVES.contains(&second.to_lowercase().as_str()) {
            return Line::Statement {
                label: Some(Label {
                    name: first.to_string(),
                    has_colon: false,
                }),
                body: optional_body(rest_trimmed),
                comment,
            };
        }
        return Line::Statement {
            label: None,
            body: Some(make_body(first, rest)),
            comment,
        };
    }

    if SECTION_DIRECTIVES.contains(&first_lower.as_str()) {
        return Line::Statement {
            label: None,
            body: Some(make_body(first, rest)),
            comment,
        };
    }

    if first.ends_with(':') {
        let name = first[..first.len() - 1].to_string();
        return Line::Statement {
            label: Some(Label {
                name,
                has_colon: true,
            }),
            body: optional_body(rest),
            comment,
        };
    }

    Line::Statement {
        label: Some(Label {
            name: first.to_string(),
            has_colon: false,
        }),
        body: optional_body(rest),
        comment,
    }
}

fn split_first_token(s: &str) -> (&str, &str) {
    let end = s.find(|c: char| c.is_whitespace()).unwrap_or(s.len());
    (&s[..end], &s[end..])
}

fn make_body(first: &str, rest: &str) -> Body {
    let mnemonic = first.to_lowercase();
    let operands = if rest.trim().is_empty() {
        vec![]
    } else {
        split_operands(rest.trim())
    };
    Body { mnemonic, operands }
}

fn make_body_str(s: &str) -> Body {
    let (first, rest) = split_first_token(s);
    make_body(first, rest)
}

fn optional_body(rest: &str) -> Option<Body> {
    let trimmed = rest.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(make_body_str(trimmed))
    }
}

fn split_operands(s: &str) -> Vec<String> {
    let mut operands = vec![];
    let mut current = String::new();
    let mut depth = 0usize;
    let mut in_string = false;
    let mut string_char = '"';
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        if in_string {
            current.push(c);
            if c == string_char {
                in_string = false;
            } else if c == '\\' && string_char != '\''
                && let Some(&next) = chars.peek() {
                    current.push(next);
                    chars.next();
                }
        } else {
            match c {
                '"' | '\'' | '`' => {
                    in_string = true;
                    string_char = c;
                    current.push(c);
                }
                '[' | '(' => {
                    depth += 1;
                    current.push(c);
                }
                ']' | ')' => {
                    depth = depth.saturating_sub(1);
                    current.push(c);
                }
                ',' if depth == 0 => {
                    operands.push(current.trim().to_string());
                    current = String::new();
                }
                _ => current.push(c),
            }
        }
    }

    let last = current.trim().to_string();
    if !last.is_empty() {
        operands.push(last);
    }
    operands
}
