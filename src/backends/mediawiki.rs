use async_trait::async_trait;

use super::{Backend, BackendOutput, QueryOptions};
use crate::error::{Error, Result};

pub struct MediaWikiBackend {
    site: String,
}

impl MediaWikiBackend {
    pub fn new(site: &str) -> Self {
        Self {
            site: site.to_string(),
        }
    }

    fn base_url(&self) -> Option<String> {
        crate::config::Config::global()
            .mediawiki
            .sites
            .get(&self.site)
            .cloned()
    }

    async fn api_get(&self, params: &str) -> Result<serde_json::Value> {
        let base = self.base_url().ok_or_else(|| {
            Error::Config(format!("unknown site '{}'", self.site))
        })?;
        let url = format!("{base}/api.php?format=json&{params}");
        let client = reqwest::Client::new();
        let resp = client
            .get(&url)
            .header("User-Agent", "corpora-atlas/1.0")
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await
            .map_err(|e| Error::Http(e))?;
        let data: serde_json::Value = resp.json().await.map_err(|e| Error::Http(e))?;
        Ok(data)
    }
}

#[async_trait]
impl Backend for MediaWikiBackend {
    async fn query(&self, query: &str, opts: &QueryOptions) -> Result<BackendOutput> {
        let q = urlencoding(query);

        let is_search = opts
            .select_dicts
            .as_ref()
            .map(|d| d.iter().any(|s| s == "search"))
            .unwrap_or(false);

        if is_search || opts.multi_file {
            if is_search {
                let api_params = format!(
                    "action=query&list=search&srsearch={q}&srwhat=text&srlimit=50"
                );
                let data = self.api_get(&api_params).await?;
                let total = data["query"]["searchinfo"]["totalhits"]
                    .as_u64()
                    .unwrap_or(0);
                let results = data["query"]["search"]
                    .as_array()
                    .map(|arr| {
                        arr.iter()
                            .map(|r| {
                                let title = r["title"].as_str().unwrap_or("");
                                let snippet = r["snippet"].as_str().unwrap_or("");
                                (title.to_string(), snippet.to_string())
                            })
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default();
                let mut lines = Vec::new();
                lines.push(format!("Results: {total} total (1-{})", results.len()));
                for (i, (title, snippet)) in results.iter().enumerate() {
                    let snippet = crate::html::strip::strip_html(snippet, false);
                    lines.push(format!("\n[{}] {title}", i + 1));
                    lines.push(snippet);
                }
                return Ok(BackendOutput::Text(lines.join("\n")));
            }

            let api_params = format!("action=query&list=allpages&aplimit=10&apfrom={q}");
            let data = self.api_get(&api_params).await?;
            let pages = data["query"]["allpages"]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .filter_map(|p| p["title"].as_str().map(|s| s.to_string()))
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            let mut lines = Vec::new();
            lines.push(format!("Prefix results: {}", pages.len()));
            for t in &pages {
                lines.push(format!("  {t}"));
            }
            return Ok(BackendOutput::Text(lines.join("\n")));
        }

        let api_params = format!("action=parse&prop=text&redirects&page={q}");
        let data = self.api_get(&api_params).await?;
        let html = data["parse"]["text"]["*"]
            .as_str()
            .unwrap_or("")
            .to_string();

        if html.is_empty() {
            return Ok(BackendOutput::Text(String::new()));
        }

        if opts.as_html {
            Ok(BackendOutput::Html(html))
        } else {
            let text = crate::html::lean::lean_text(&html, None);
            Ok(BackendOutput::Text(text))
        }
    }
}

fn urlencoding(text: &str) -> String {
    text.bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                (b as char).to_string()
            }
            b' ' => "_".to_string(),
            _ => format!("%{:02X}", b),
        })
        .collect()
}
