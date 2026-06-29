use unicode_categories::UnicodeCategories;

const MARKDOWN_CHARS: &[char] = &['*', '_', '~', '`', '#', '='];

pub fn strip_query(text: &str) -> String {
    let text: String = text.chars()
        .filter(|c| !MARKDOWN_CHARS.contains(c))
        .collect();
    let trimmed_start = text
        .chars()
        .skip_while(|c| c.is_punctuation() || c.is_symbol() || c.is_whitespace())
        .collect::<String>();
    let trimmed_end = trimmed_start
        .chars()
        .rev()
        .skip_while(|c| c.is_punctuation() || c.is_symbol() || c.is_whitespace())
        .collect::<String>();
    trimmed_end.chars().rev().collect()
}
