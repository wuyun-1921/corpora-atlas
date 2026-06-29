use regex::Regex;

static STYLE_RE: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    Regex::new(r"(?i)<style[^>]*>.*?</style>").unwrap()
});
static SCRIPT_RE: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    Regex::new(r"(?i)<script[^>]*>.*?</script>").unwrap()
});
static BLOCK_RE: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    Regex::new(r"(?i)</?(?:div|p|br|hr|li|tr|h[1-6]|blockquote|section|header|footer|nav)[^>]*>")
        .unwrap()
});
static TAG_RE: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    Regex::new(r"<[^>]+>").unwrap()
});
static NEWLINES_RE: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    Regex::new(r"\n{3,}").unwrap()
});
static MULTISPACE_RE: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    Regex::new(r"\s+").unwrap()
});

pub fn strip_html(text: &str, block_tags: bool) -> String {
    let text = STYLE_RE.replace_all(text, "");
    let text = SCRIPT_RE.replace_all(&text, "");
    if block_tags {
        let text = BLOCK_RE.replace_all(&text, "\n");
        let text = TAG_RE.replace_all(&text, "");
        let text = html_escape::decode_html_entities(&text);
        let text = NEWLINES_RE.replace_all(&text, "\n\n");
        let text = text
            .lines()
            .map(|l| l.trim())
            .collect::<Vec<_>>()
            .join("\n");
        return text.trim().to_string();
    }
    let text = TAG_RE.replace_all(&text, " ");
    let text = html_escape::decode_html_entities(&text);
    let text = MULTISPACE_RE.replace_all(&text, " ");
    text.trim().to_string()
}
