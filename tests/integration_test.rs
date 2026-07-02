use corpora_atlas::lang::{self, Script};
use corpora_atlas::state::DaemonState;
use corpora_atlas::strip;
use corpora_atlas::html;

#[test]
fn test_strip_query_removes_markdown() {
    assert_eq!(strip::strip_query("*hello*"), "hello");
    assert_eq!(strip::strip_query("_world_"), "world");
    assert_eq!(strip::strip_query("~~strike~~"), "strike");
    assert_eq!(strip::strip_query("`code`"), "code");
    assert_eq!(strip::strip_query("#hash"), "hash");
    assert_eq!(strip::strip_query("=tag="), "tag");
}

#[test]
fn test_strip_query_trims_punct() {
    assert_eq!(strip::strip_query("\"...hello...\""), "hello");
    assert_eq!(strip::strip_query("「こんにちは」"), "こんにちは");
    assert_eq!(strip::strip_query("  spaced  "), "spaced");
}

#[test]
fn test_strip_query_empty() {
    assert_eq!(strip::strip_query(""), "");
    assert_eq!(strip::strip_query("***"), "");
}

#[test]
fn test_lang_detect_english() {
    assert_eq!(Script::detect("hello"), Script::Latin);
    assert_eq!(Script::detect("World"), Script::Latin);
    assert_eq!(Script::detect("test123"), Script::Latin);
}

#[test]
fn test_lang_detect_chinese() {
    assert_eq!(Script::detect("你好"), Script::Chinese);
    assert_eq!(Script::detect("中文"), Script::Chinese);
}

#[test]
fn test_lang_detect_japanese() {
    assert_eq!(Script::detect("こんにちは"), Script::Japanese);
    assert_eq!(Script::detect("カタカナ"), Script::Japanese);
}

#[test]
fn test_lang_detect_hangul() {
    assert_eq!(Script::detect("안녕하세요"), Script::Hangul);
}

#[test]
fn test_lang_detect_cyrillic() {
    assert_eq!(Script::detect("Привет"), Script::Cyrillic);
}

#[test]
fn test_lang_detect_greek() {
    assert_eq!(Script::detect("γειά"), Script::Greek);
}

#[test]
fn test_lang_is_alpha() {
    assert!(lang::is_alpha("hello"));
    assert!(lang::is_alpha("你好"));
    assert!(!lang::is_alpha("😀"));
    assert!(!lang::is_alpha("123"));
}

#[test]
fn test_html_strip_block_tags() {
    let html = "<div><p>Hello</p><p>World</p></div>";
    let result = html::strip::strip_html(html, true);
    assert_eq!(result, "Hello\n\nWorld");
}

#[test]
fn test_html_strip_inline() {
    let html = "<p>Hello <b>World</b></p>";
    let result = html::strip::strip_html(html, false);
    assert_eq!(result, "Hello World");
}

#[test]
fn test_html_strip_removes_scripts() {
    let html = "<p>Hello</p><script>alert('x')</script><p>World</p>";
    let result = html::strip::strip_html(html, false);
    assert!(result.contains("Hello"));
    assert!(result.contains("World"));
    assert!(!result.contains("alert"));
}

#[test]
fn test_html_strip_removes_styles() {
    let html = "<p>Hello</p><style>body{color:red}</style><p>World</p>";
    let result = html::strip::strip_html(html, false);
    assert!(!result.contains("color"));
}

#[test]
fn test_toc_extraction_empty() {
    assert!(html::lean::extract_toc("<html></html>").is_none());
}

#[test]
fn test_toc_format_empty() {
    let result = html::lean::format_toc(&[], 0);
    assert_eq!(result, "");
}

#[test]
fn test_toc_format_single() {
    let toc = vec![html::lean::TocItem {
        id: "section1".into(),
        title: "Section One".into(),
        children: vec![],
    }];
    let result = html::lean::format_toc(&toc, 0);
    assert_eq!(result, "#section1  Section One");
}

#[test]
fn test_toc_format_nested() {
    let toc = vec![html::lean::TocItem {
        id: "s1".into(),
        title: "S1".into(),
        children: vec![html::lean::TocItem {
            id: "s1_1".into(),
            title: "S1.1".into(),
            children: vec![],
        }],
    }];
    let result = html::lean::format_toc(&toc, 0);
    assert!(result.contains("s1"));
    assert!(result.contains("s1_1"));
}

#[test]
fn test_extract_title_missing() {
    assert!(html::lean::extract_title("<html></html>").is_empty());
}

#[test]
fn test_lean_text_no_section() {
    let html = "<p>Hello World</p>";
    let result = html::lean::lean_text(html, None);
    assert_eq!(result, "Hello World");
}

#[test]
fn test_lean_text_unknown_section() {
    let result = html::lean::lean_text("<p>Hello</p>", Some("nosuch"));
    assert_eq!(result, "Section '#nosuch' not found.");
}

#[test]
fn test_extract_section_lead_empty() {
    let result = html::lean::extract_section("<h2>Start</h2>", "_lead");
    assert_eq!(result, "");
}

#[test]
fn test_state_defaults() {
    let state = DaemonState::default();
    assert_eq!(state.repeat, 0);
    assert!(!state.monitoring);
    assert!(!state.focus_gd);
    assert_eq!(state.prev_query, "");
}

#[test]
fn test_state_advance_new_query() {
    let mut state = DaemonState::default();
    let chain = vec!["A".to_string(), "B".to_string(), "C".to_string()];
    let repeat = state.advance("hello", &chain);
    assert_eq!(repeat, 0);
    assert_eq!(state.group, "A");
}

#[test]
fn test_state_advance_repeat_query() {
    let mut state = DaemonState::default();
    let chain = vec!["A".to_string(), "B".to_string()];
    state.advance("hello", &chain);
    let repeat = state.advance("hello", &chain);
    assert_eq!(repeat, 1);
    assert_eq!(state.group, "B");
}

#[test]
fn test_state_advance_cycles() {
    let mut state = DaemonState::default();
    let chain = vec!["A".to_string(), "B".to_string()];
    state.advance("hello", &chain);
    state.advance("hello", &chain);
    let repeat = state.advance("hello", &chain);
    assert_eq!(repeat, 2);
    assert_eq!(state.group, "A");
}

#[test]
fn test_state_mark_done() {
    let mut state = DaemonState::default();
    state.mark_done("test", "GRP");
    assert_eq!(state.prev_query, "test");
    assert_eq!(state.repeat, 0);
    assert_eq!(state.group, "GRP");
}

#[test]
fn test_state_save_load_roundtrip() {
    let mut state = DaemonState::default();
    state.prev_query = "test".into();
    state.repeat = 3;
    state.group = "ZH".into();
    state.monitoring = true;

    let json = serde_json::to_string_pretty(&state).unwrap();
    let loaded: DaemonState = serde_json::from_str(&json).unwrap();
    assert_eq!(loaded.prev_query, "test");
    assert_eq!(loaded.repeat, 3);
    assert_eq!(loaded.group, "ZH");
    assert!(loaded.monitoring);
}
