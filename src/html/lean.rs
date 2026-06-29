use regex::Regex;
use scraper::{Html, Selector};

use super::strip::strip_html;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TocItem {
    pub id: String,
    pub title: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<TocItem>,
}

static TOC_RE_A2: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    Regex::new(r#"<div id="a2-toc">\s*<ol class="toc">(.*?)</ol>\s*</div>"#).unwrap()
});
static TOC_RE_PLAIN: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    Regex::new(r#"<ol class="toc">(.*?)</ol>"#).unwrap()
});
static TAG_RE: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    Regex::new(r"<[^>]+>").unwrap()
});
static HEADING_RE: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    Regex::new(r#"<(?:h[2-3])[^>]*id="([^"]+)"[^>]*>(.*?)</(?:h[2-3])>"#).unwrap()
});

fn strip_html_lean(text: &str) -> String {
    let text = TAG_RE.replace_all(text, " ");
    let text = html_escape::decode_html_entities(&text);
    let text: Vec<&str> = text.split_whitespace().collect();
    text.join(" ")
}

pub fn extract_toc(html: &str) -> Option<Vec<TocItem>> {
    if let Some(caps) = TOC_RE_A2.captures(html).or_else(|| TOC_RE_PLAIN.captures(html)) {
        let inner = caps.get(1).unwrap().as_str();
        let parsed = Html::parse_fragment(inner);

        if let Ok(sel_a) = Selector::parse("li") {
            let items = extract_toc_items(&parsed, &sel_a);
            if !items.is_empty() {
                return Some(items);
            }
        }
    }

    let headings: Vec<TocItem> = HEADING_RE
        .captures_iter(html)
        .map(|c| TocItem {
            id: c.get(1).unwrap().as_str().to_string(),
            title: strip_html_lean(c.get(2).unwrap().as_str()),
            children: vec![],
        })
        .collect();
    if !headings.is_empty() {
        Some(headings)
    } else {
        None
    }
}

fn extract_toc_items(doc: &Html, sel: &Selector) -> Vec<TocItem> {
    let mut items = Vec::new();
    let a_sel = Selector::parse("a").ok();
    let ol_sel = Selector::parse("ol li").ok();
    for li_elem in doc.select(sel) {
        let a_sel = match &a_sel {
            Some(s) => s,
            None => continue,
        };
        if let Some(a_elem) = li_elem.select(a_sel).next() {
            let href = a_elem.value().attr("href").unwrap_or("");
            let id = href.trim_start_matches('#');
            let title: String = a_elem.text().collect::<Vec<_>>().join(" ");
            let title = strip_html_lean(&title);
            let children = match &ol_sel {
                Some(s) => {
                    let inner = li_elem.html();
                    let parsed = Html::parse_fragment(&inner);
                    extract_toc_items(&parsed, s)
                }
                None => vec![],
            };
            items.push(TocItem {
                id: id.to_string(),
                title,
                children,
            });
        }
    }
    items
}

fn section_from_heading(html: &str, section_id: &str) -> String {
    let pattern = match Regex::new(
        &format!(
            r#"<h[2-6]\s[^>]*id="{}"[^>]*>.*?</h[2-6]>(.*?)(?=<h[2-6]\s[^>]*id="|<section[^>]*>|</section>\s*$|</body>|</div>\s*$)"#,
            regex::escape(section_id)
        )
    ) {
        Ok(r) => r,
        Err(_) => return String::new(),
    };
    if let Some(caps) = pattern.captures(html) {
        return caps.get(1).map_or("", |m| m.as_str()).trim().to_string();
    }
    String::new()
}

pub fn extract_section(html: &str, section_id: &str) -> String {
    if section_id == "_lead" {
        let re = match Regex::new(
            r"(?:<div[^>]*>\s*)?(.*?)(?:<h[2-6]\s|<section[^>]*>\s*<h[2-6]\s)"
        ) {
            Ok(r) => r,
            Err(_) => return String::new(),
        };
        if let Some(caps) = re.captures(html) {
            return caps.get(1).map_or("", |m| m.as_str()).trim().to_string();
        }
        return String::new();
    }

    let result = section_from_heading(html, section_id);
    if !result.is_empty() {
        return result;
    }

    let pattern = match Regex::new(
        &format!(
            r#"<section[^>]*>.*?<h[2-6]\s[^>]*id="{}"[^>]*>.*?</h[2-6]>(.*?)(?=<section[^>]*>|</section>\s*$|</body>)"#,
            regex::escape(section_id)
        )
    ) {
        Ok(r) => r,
        Err(_) => return String::new(),
    };
    if let Some(caps) = pattern.captures(html) {
        return caps.get(1).map_or("", |m| m.as_str()).trim().to_string();
    }
    String::new()
}

pub fn extract_title(html: &str) -> String {
    let pattern = match Regex::new(r#"<span id="a2-title"[^>]*>(.*?)</span>"#) {
        Ok(r) => r,
        Err(_) => return String::new(),
    };
    if let Some(caps) = pattern.captures(html) {
        return strip_html_lean(caps.get(1).map_or("", |m| m.as_str()));
    }
    String::new()
}

pub fn format_toc(toc: &[TocItem], indent: usize) -> String {
    let mut lines = Vec::new();
    for item in toc {
        let prefix = if indent > 0 {
            format!("{:indent$}", "", indent = indent * 2)
        } else {
            String::new()
        };
        let marker = if indent > 0 { "- " } else { "" };
        lines.push(format!("{}{}#{}  {}", prefix, marker, item.id, item.title));
        if !item.children.is_empty() {
            lines.push(format_toc(&item.children, indent + 1));
        }
    }
    lines.join("\n")
}

pub fn lean_text(html: &str, section_id: Option<&str>) -> String {
    if let Some(sid) = section_id {
        let section_html = extract_section(html, sid);
        if section_html.is_empty() {
            return format!("Section '#{sid}' not found.");
        }
        return strip_html(&section_html, false);
    }
    strip_html(html, false)
}
