use super::util::rtrim;

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

pub(super) fn format_operands(operands: &[String]) -> String {
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

fn consume_string_literal<I>(chars: &mut std::iter::Peekable<I>, result: &mut String, quote: char)
where
    I: Iterator<Item = (usize, char)>,
{
    while let Some((_, nc)) = chars.next() {
        result.push(nc);
        if nc == quote {
            break;
        }
        if nc == '\\'
            && quote != '\''
            && let Some((_, ec)) = chars.next()
        {
            result.push(ec);
        }
    }
}

fn format_mem_expr(s: &str) -> String {
    let mut result = String::new();
    let mut chars = s.char_indices().peekable();
    while let Some((i, c)) = chars.next() {
        match c {
            '"' | '\'' | '`' => {
                result.push(c);
                consume_string_literal(&mut chars, &mut result, c);
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
                // Preserve space between adjacent word tokens (e.g. `rel msg`)
                if result
                    .chars()
                    .last()
                    .is_some_and(|lc| lc.is_alphanumeric() || matches!(lc, '_' | '$' | '.' | '@'))
                {
                    result.push(' ');
                }
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

#[cfg(test)]
mod tests {
    use super::*;

    fn ops(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| s.to_string()).collect()
    }

    // --- format_token ---

    #[test]
    fn token_register_lowercased() {
        assert_eq!(format_token("RAX"), "rax");
    }

    #[test]
    fn token_already_lowercase_register() {
        assert_eq!(format_token("rax"), "rax");
    }

    #[test]
    fn token_non_register_symbol_preserved() {
        assert_eq!(format_token("MyLabel"), "MyLabel");
    }

    #[test]
    fn token_size_prefix_lowercased() {
        assert_eq!(format_token("BYTE"), "byte");
    }

    #[test]
    fn token_number_unchanged() {
        assert_eq!(format_token("42"), "42");
    }

    #[test]
    fn token_hex_lowercase_unchanged() {
        assert_eq!(format_token("0xff"), "0xff");
    }

    // --- format_operands ---

    #[test]
    fn operands_empty_list() {
        assert_eq!(format_operands(&[]), "");
    }

    #[test]
    fn operands_single_register() {
        assert_eq!(format_operands(&ops(&["rax"])), "rax");
    }

    #[test]
    fn operands_two_registers() {
        assert_eq!(format_operands(&ops(&["rax", "rbx"])), "rax, rbx");
    }

    #[test]
    fn operands_three_items() {
        assert_eq!(
            format_operands(&ops(&["rax", "rbx", "rcx"])),
            "rax, rbx, rcx"
        );
    }

    #[test]
    fn operands_uppercase_register_lowercased() {
        assert_eq!(format_operands(&ops(&["RAX", "RBX"])), "rax, rbx");
    }

    #[test]
    fn operands_non_register_symbol_preserved() {
        assert_eq!(format_operands(&ops(&["MyLabel"])), "MyLabel");
    }

    // --- size prefix ---

    #[test]
    fn operands_size_prefix_byte() {
        assert_eq!(format_operands(&ops(&["byte [rax]"])), "byte [rax]");
    }

    #[test]
    fn operands_size_prefix_qword() {
        assert_eq!(
            format_operands(&ops(&["qword [rbp - 8]"])),
            "qword [rbp - 8]"
        );
    }

    #[test]
    fn operands_size_prefix_uppercase_lowercased() {
        assert_eq!(format_operands(&ops(&["BYTE [RAX]"])), "byte [rax]");
    }

    // --- memory references ---

    #[test]
    fn operands_plain_memory_ref() {
        assert_eq!(format_operands(&ops(&["[rax]"])), "[rax]");
    }

    #[test]
    fn operands_memory_arithmetic_adds_spaces() {
        assert_eq!(format_operands(&ops(&["[rbp-8]"])), "[rbp - 8]");
    }

    #[test]
    fn operands_memory_multiplication() {
        assert_eq!(format_operands(&ops(&["[rax+rbx*4]"])), "[rax + rbx * 4]");
    }

    #[test]
    fn operands_memory_complex_expression() {
        assert_eq!(format_operands(&ops(&["[rbp - 8]"])), "[rbp - 8]");
    }

    #[test]
    fn operands_unary_minus() {
        assert_eq!(format_operands(&ops(&["-1"])), "-1");
    }

    // --- adjacent tokens in memory (rel, abs, nosplit) ---

    #[test]
    fn operands_rel_keyword_space_preserved() {
        assert_eq!(format_operands(&ops(&["[rel msg]"])), "[rel msg]");
    }

    #[test]
    fn operands_abs_keyword_space_preserved() {
        assert_eq!(format_operands(&ops(&["[abs label]"])), "[abs label]");
    }

    // --- string literals ---

    #[test]
    fn operands_double_quoted_string() {
        assert_eq!(format_operands(&ops(&["\"Hello\""])), "\"Hello\"");
    }

    #[test]
    fn operands_string_with_uppercase_not_changed() {
        assert_eq!(format_operands(&ops(&["\"HELLO\""])), "\"HELLO\"");
    }
}
