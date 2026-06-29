use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("config: {0}")]
    Config(String),
    #[error("config not found at {0}")]
    ConfigNotFound(PathBuf),
    #[error("I/O: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON: {0}")]
    Json(#[from] serde_json::Error),
    #[error("YAML: {0}")]
    Yaml(#[from] serde_yaml::Error),
    #[error("HTTP: {0}")]
    Http(#[from] reqwest::Error),
    #[error("WebSocket: {0}")]
    Ws(#[from] tokio_tungstenite::tungstenite::Error),
    #[error("CDP: {0}")]
    Cdp(String),
    #[error("{0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, Error>;
