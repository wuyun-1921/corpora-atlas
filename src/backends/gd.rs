use std::collections::HashMap;
use std::path::PathBuf;

use async_trait::async_trait;
use futures_util::SinkExt;
use futures_util::StreamExt;
use regex::Regex;
use serde::Deserialize;
use tokio_tungstenite::connect_async;

use super::{Backend, BackendOutput, QueryOptions};
use crate::error::{Error, Result};
use crate::html::strip::strip_html;

pub struct GdBackend;

#[derive(Debug, Deserialize)]
struct CdpTarget {
    id: String,
    #[serde(default)]
    url: String,
}

impl GdBackend {
    fn config(&self) -> &'static crate::config::GdConfig {
        &crate::config::Config::global().gd
    }

    fn cdp_port(&self) -> u16 {
        let addr = &crate::config::Config::global().servers.gd_cdp;
        addr.split(':')
            .last()
            .and_then(|p| p.parse().ok())
            .unwrap_or(18123)
    }

    async fn discover_targets(&self) -> Result<Vec<CdpTarget>> {
        let url = format!("http://localhost:{}/json/list", self.cdp_port());
        let client = reqwest::Client::new();
        let resp = client
            .get(&url)
            .timeout(std::time::Duration::from_secs(self.config().cdp_timeout))
            .send()
            .await
            .map_err(|e| Error::Http(e))?;
        let targets: Vec<CdpTarget> = resp.json().await.map_err(|e| Error::Http(e))?;
        Ok(targets)
    }

    fn find_gdlookup<'a>(targets: &'a [CdpTarget]) -> Option<&'a str> {
        targets
            .iter()
            .find(|t| t.url.contains("gdlookup"))
            .map(|t| t.id.as_str())
    }

    async fn extract_page(&self, target_id: &str) -> Result<String> {
        let ws_url = format!("ws://localhost:{}/devtools/page/{target_id}", self.cdp_port());
        let (ws, _) = connect_async(&ws_url)
            .await
            .map_err(|e| Error::Cdp(format!("WS connect failed: {e}")))?;

        let (mut write, mut read) = ws.split();

        let enable_req = serde_json::json!({"id": 1, "method": "Runtime.enable"});
        write
            .send(tokio_tungstenite::tungstenite::Message::Text(
                serde_json::to_string(&enable_req).unwrap().into(),
            ))
            .await
            .map_err(|e| Error::Cdp(format!("WS send failed: {e}")))?;

        loop {
            let msg = read
                .next()
                .await
                .ok_or_else(|| Error::Cdp("no response for Runtime.enable".into()))?
                .map_err(|e| Error::Cdp(format!("WS recv failed: {e}")))?;
            let text = msg.to_text().unwrap_or("{}").to_string();
            let v: serde_json::Value =
                serde_json::from_str(&text).map_err(|e| Error::Cdp(format!("JSON parse: {e}")))?;
            if v.get("id") == Some(&serde_json::json!(1)) {
                break;
            }
        }

        let eval_req = serde_json::json!({
            "id": 2,
            "method": "Runtime.evaluate",
            "params": {
                "expression": "document.body.innerHTML",
                "returnByValue": true,
            },
        });
        write
            .send(tokio_tungstenite::tungstenite::Message::Text(
                serde_json::to_string(&eval_req).unwrap().into(),
            ))
            .await
            .map_err(|e| Error::Cdp(format!("WS send failed: {e}")))?;

        loop {
            let msg = read
                .next()
                .await
                .ok_or_else(|| Error::Cdp("no response for Runtime.evaluate".into()))?
                .map_err(|e| Error::Cdp(format!("WS recv failed: {e}")))?;
            let text = msg.to_text().unwrap_or("{}").to_string();
            let v: serde_json::Value =
                serde_json::from_str(&text).map_err(|e| Error::Cdp(format!("JSON parse: {e}")))?;
            if v.get("id") == Some(&serde_json::json!(2)) {
                let body = v["result"]["result"]["value"]
                    .as_str()
                    .unwrap_or("")
                    .to_string();
                return Ok(body);
            }
        }
    }

    fn split_articles(raw_html: &str) -> Vec<(String, String)> {
        let mut results = Vec::new();
        let parts: Vec<&str> = raw_html.splitn(2, r#"<article class="gdarticle"#).collect();
        if parts.len() < 2 {
            return results;
        }
        let rest = parts[1];
        for chunk in rest.split(r#"<article class="gdarticle"#) {
            let title_re = Regex::new(r#"<span class="gddicttitle">(.*?)</span>"#).unwrap();
            let name = title_re
                .captures(chunk)
                .and_then(|c| c.get(1))
                .map(|m| html_escape::decode_html_entities(m.as_str()).to_string())
                .unwrap_or_else(|| "unknown".to_string());
            let sec_start = chunk.find("<section").unwrap_or(0);
            let content = chunk[sec_start..].to_string();
            results.push((name, content));
        }
        results
    }

    fn parse_group_map(&self) -> HashMap<String, String> {
        let mut map = HashMap::new();
        let config_path: PathBuf = self.config().config_path.clone();
        if !config_path.exists() {
            return map;
        }
        let content = match std::fs::read_to_string(&config_path) {
            Ok(c) => c,
            Err(_) => return map,
        };
        let re = Regex::new(r#"<group\s+id="(\d+)"\s+name="([^"]*)"\s*/>"#).unwrap();
        for cap in re.captures_iter(&content) {
            map.insert(cap[1].to_string(), cap[2].to_string());
        }
        map
    }

    pub async fn get_current_group(&self) -> Option<String> {
        let targets = self.discover_targets().await.ok()?;
        for t in &targets {
            if !t.url.contains("gdlookup") {
                continue;
            }
            let re = Regex::new(r"group=(\d+)").ok()?;
            if let Some(cap) = re.captures(&t.url) {
                let group_id = cap.get(1)?.as_str();
                let map = self.parse_group_map();
                return map.get(group_id).cloned();
            }
        }
        None
    }

    fn launch_gd(&self, word: &str, group: Option<&str>) {
        let mut cmd = std::process::Command::new(&self.config().binary);
        if let Some(g) = group {
            cmd.arg("-g").arg(g);
        }
        cmd.arg(word);
        let _ = cmd
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn();
    }

    fn wait_for_gd(&self, group: Option<&str>) {
        let wait = if let Some(g) = group {
            if g == "H" { 3.0 } else { 1.5 }
        } else {
            1.5
        };
        std::thread::sleep(std::time::Duration::from_secs_f64(wait));
    }
}

#[async_trait]
impl Backend for GdBackend {
    async fn query(&self, query: &str, opts: &QueryOptions) -> Result<BackendOutput> {
        self.launch_gd(query, opts.group.as_deref());
        self.wait_for_gd(opts.group.as_deref());

        let targets = self.discover_targets().await?;
        let target_id = GdBackend::find_gdlookup(&targets)
            .map(|s| s.to_string())
            .ok_or_else(|| Error::Cdp("gdlookup page not found".into()))?;

        let raw_html = self.extract_page(&target_id).await?;
        let articles = GdBackend::split_articles(&raw_html);

        if !opts.extract_all && opts.select_dicts.is_none() && !opts.multi_file {
            let lines: Vec<String> = articles
                .iter()
                .enumerate()
                .map(|(i, (name, _))| format!("  {:2}. {name}", i + 1))
                .collect();
            return Ok(BackendOutput::Text(lines.join("\n")));
        }

        let mut results = HashMap::new();
        for (name, chunk) in &articles {
            if let Some(ref dicts) = opts.select_dicts {
                let matched =
                    dicts.iter().any(|d| name.to_lowercase().contains(&d.to_lowercase()));
                if !opts.extract_all && !matched {
                    continue;
                }
            }
            let text = if opts.as_html {
                chunk.clone()
            } else {
                strip_html(chunk, true)
            };
            let entry = if opts.anchors && !opts.as_html {
                format!("# From {name}\n\n{text}")
            } else {
                text
            };
            results.insert(name.clone(), entry);
        }

        Ok(BackendOutput::Multi(results))
    }
}
