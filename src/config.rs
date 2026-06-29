use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::OnceLock;

use serde::Deserialize;

use crate::error::{Error, Result};

#[derive(Debug, Deserialize)]
pub struct Paths {
    pub socket: PathBuf,
    pub state: PathBuf,
    pub lock: PathBuf,
    pub pid: PathBuf,
    pub output: PathBuf,
}

#[derive(Debug, Deserialize)]
pub struct Servers {
    pub gd_cdp: String,
    pub kiwix: String,
    pub aard2: String,
}

#[derive(Debug, Deserialize)]
pub struct WebUi {
    pub port: u16,
    pub host: String,
}

#[derive(Debug, Deserialize)]
pub struct DaemonConfig {
    pub poll_interval: f64,
    pub max_query_len: usize,
}

#[derive(Debug, Deserialize)]
pub struct GdConfig {
    pub binary: String,
    pub config_path: PathBuf,
    pub cdp_timeout: u64,
    #[serde(default = "default_window_app_id")]
    pub window_app_id: String,
}

fn default_window_app_id() -> String {
    "goldendict".to_string()
}

#[derive(Debug, Deserialize)]
pub struct MediawikiConfig {
    pub sites: HashMap<String, String>,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub paths: Paths,
    pub servers: Servers,
    pub webui: WebUi,
    pub daemon: DaemonConfig,
    pub gd: GdConfig,
    pub fallback: HashMap<String, Vec<String>>,
    pub kiwix: HashMap<String, String>,
    pub aard2: HashMap<String, String>,
    pub mediawiki: MediawikiConfig,
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_path = dirs_config_path();
        if !config_path.exists() {
            return Err(Error::ConfigNotFound(config_path));
        }
        let contents = std::fs::read_to_string(&config_path)
            .map_err(|e| Error::Config(format!("failed to read {}: {e}", config_path.display())))?;
        let config: Config = serde_yaml::from_str(&contents)
            .map_err(|e| Error::Config(format!("failed to parse {}: {e}", config_path.display())))?;
        config.validate()?;
        Ok(config)
    }

    pub fn init() -> Result<()> {
        let config = Config::load()?;
        CONFIG.set(config).map_err(|_| Error::Config("config already initialized".into()))
    }

    pub fn global() -> &'static Config {
        CONFIG.get().expect("config not initialized — call Config::init() first")
    }

    fn validate(&self) -> Result<()> {
        if self.daemon.poll_interval <= 0.0 {
            return Err(Error::Config("daemon.poll_interval must be > 0".into()));
        }
        if self.gd.cdp_timeout == 0 {
            return Err(Error::Config("gd.cdp_timeout must be > 0".into()));
        }
        Ok(())
    }
}

static CONFIG: OnceLock<Config> = OnceLock::new();

fn dirs_config_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("~/.config"))
        .join("corpora-atlas")
        .join("config.yaml")
}
