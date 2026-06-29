use std::collections::HashMap;

use async_trait::async_trait;

use crate::error::Result;

#[derive(Debug, Clone)]
pub struct QueryOptions {
    pub as_html: bool,
    pub group: Option<String>,
    pub select_dicts: Option<Vec<String>>,
    pub extract_all: bool,
    pub multi_file: bool,
    pub anchors: bool,
}

impl Default for QueryOptions {
    fn default() -> Self {
        Self {
            as_html: false,
            group: None,
            select_dicts: None,
            extract_all: false,
            multi_file: false,
            anchors: false,
        }
    }
}

#[derive(Debug)]
pub enum BackendOutput {
    Text(String),
    Html(String),
    Multi(HashMap<String, String>),
}

#[async_trait]
pub trait Backend: Send + Sync {
    async fn query(&self, query: &str, opts: &QueryOptions) -> Result<BackendOutput>;
}

pub mod gd;
pub mod kiwix;
pub mod aard2;
pub mod mediawiki;
