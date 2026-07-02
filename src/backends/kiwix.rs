use async_trait::async_trait;
use regex::Regex;

use super::{Backend, BackendOutput, QueryOptions};
use crate::error::{Error, Result};
use crate::html::strip::strip_html;

pub struct KiwixBackend {
    zim_name: String,
}

impl KiwixBackend {
    /// `zim_name` is expected to already be resolved (caller handled shorthand lookup).
    pub fn new(zim_name: &str) -> Self {
        Self {
            zim_name: zim_name.to_string(),
        }
    }
}

#[async_trait]
impl Backend for KiwixBackend {
    async fn query(&self, query: &str, opts: &QueryOptions) -> Result<BackendOutput> {
        let base = &crate::config::Config::global().servers.kiwix;
        let q = urlencoding(&query);
        let url = format!(
            "{base}/search?pattern={q}&books.name={}&pageLength=25&page={}",
            self.zim_name, 1
        );
        let client = reqwest::Client::new();
        let resp = client
            .get(&url)
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await
            .map_err(|e| Error::Http(e))?;
        let content = resp.text().await.map_err(|e| Error::Http(e))?;

        if opts.as_html {
            return Ok(BackendOutput::Html(content));
        }

        let count_re = Regex::new(r"of <b>(\d+)</b>").unwrap();
        let total = count_re
            .captures(&content)
            .and_then(|c| c.get(1))
            .and_then(|m| m.as_str().parse::<usize>().ok())
            .unwrap_or(0);

        let snippets: Vec<String> = Regex::new(r"<cite>(.*?)</cite>")
            .unwrap()
            .captures_iter(&content)
            .take(25)
            .filter_map(|c| {
                let s = strip_html(c.get(1).map_or("", |m| m.as_str()), false);
                if s.is_empty() { None } else { Some(s) }
            })
            .collect();

        let mut lines = Vec::new();
        lines.push(format!("Results: {total} (page 1, 1-{})", snippets.len()));
        for (i, s) in snippets.iter().enumerate() {
            let display = if s.len() > 200 { &s[..200] } else { s.as_str() };
            lines.push(format!("\n[{}] {}", i + 1, display));
        }
        Ok(BackendOutput::Text(lines.join("\n")))
    }
}

fn urlencoding(text: &str) -> String {
    text.bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => (b as char).to_string(),
            b' ' => "+".to_string(),
            _ => format!("%{:02X}", b),
        })
        .collect()
}
