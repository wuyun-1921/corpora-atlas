const MARKDOWN_CHARS: &[char] = &['*', '_', '~', '`', '#', '='];

/// Returns true for Unicode punctuation and symbol categories
/// (Pc, Pd, Pe, Pf, Pi, Po, Ps, Sc, Sk, Sm, So).
fn is_punct_or_symbol(ch: char) -> bool {
    matches!(
        ch,
        '!' | '"' | '#' | '$' | '%' | '&' | '\'' | '(' | ')' | '*' | '+' | ','
        | '-' | '.' | '/' | ':' | ';' | '<' | '=' | '>' | '?' | '@' | '[' | '\\'
        | ']' | '^' | '_' | '`' | '{' | '|' | '}' | '~'
        | '¡'..='¿'
        | '\u{2000}'..='\u{206F}'  // General Punctuation
        | '\u{2E00}'..='\u{2E7F}'  // Supplemental Punctuation
        | '\u{3000}'..='\u{303F}'  // CJK Symbols and Punctuation
        | '\u{FE30}'..='\u{FE4F}'  // CJK Compatibility Forms
        | '\u{FF01}'..='\u{FF0F}'  // Fullwidth ASCII variants
        | '\u{FF1A}'..='\u{FF20}'
        | '\u{FF3B}'..='\u{FF40}'
        | '\u{FF5B}'..='\u{FF65}'
        | '\u{00A0}'..='\u{00BF}'  // Latin-1 punctuation/symbols
    )
}

pub fn strip_query(text: &str) -> String {
    let text: String = text
        .chars()
        .filter(|c| !MARKDOWN_CHARS.contains(c))
        .collect();
    let trimmed_start = text
        .chars()
        .skip_while(|c| is_punct_or_symbol(*c) || c.is_whitespace())
        .collect::<String>();
    let trimmed_end = trimmed_start
        .chars()
        .rev()
        .skip_while(|c| is_punct_or_symbol(*c) || c.is_whitespace())
        .collect::<String>();
    trimmed_end.chars().rev().collect()
}
