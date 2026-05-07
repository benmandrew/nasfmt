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
        if nc == '\\' && quote != '\''
            && let Some((_, ec)) = chars.next() {
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
