use async_trait::async_trait;

use super::{Backend, BackendOutput, QueryOptions};
use crate::error::{Error, Result};

pub struct Aard2Backend;

impl Aard2Backend {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Backend for Aard2Backend {
    async fn query(&self, query: &str, opts: &QueryOptions) -> Result<BackendOutput> {
        let base = &crate::config::Config::global().servers.aard2;
        let q = urlencoding(query);
        let url = format!("{base}/find/?key={q}");
        let client = reqwest::Client::new();
        let resp = client
            .get(&url)
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await
            .map_err(|e| Error::Http(e))?;
        let data: Vec<serde_json::Value> = resp
            .json()
            .await
            .map_err(|e| Error::Http(e))?;

        if opts.as_html {
            if let Some(item) = data.first() {
                if let Some(article_url) = item.get("url").and_then(|u| u.as_str()) {
                    let full_url = format!("{base}{article_url}");
                    let resp = client
                        .get(&full_url)
                        .timeout(std::time::Duration::from_secs(10))
                        .send()
                        .await
                        .map_err(|e| Error::Http(e))?;
                    let html = resp.text().await.map_err(|e| Error::Http(e))?;
                    return Ok(BackendOutput::Html(html));
                }
            }
            return Ok(BackendOutput::Html("<p>No results found</p>".into()));
        }

        let mut lines = Vec::new();
        lines.push(format!("Results: {}", data.len()));
        for item in &data {
            let label = item.get("dictLabel").and_then(|v| v.as_str()).unwrap_or("?");
            let title = item.get("label").and_then(|v| v.as_str()).unwrap_or("?");
            let url = item.get("url").and_then(|v| v.as_str()).unwrap_or("");
            lines.push(format!("\n  [{label}] {title}  -> {url}"));
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
