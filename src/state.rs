use std::path::PathBuf;

use fs2::FileExt;

use crate::error::{Error, Result};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct DaemonState {
    #[serde(default)]
    pub prev_query: String,
    #[serde(default)]
    pub repeat: usize,
    #[serde(default)]
    pub group: String,
    #[serde(default)]
    pub monitoring: bool,
    #[serde(default)]
    pub focus_gd: bool,
}

impl Default for DaemonState {
    fn default() -> Self {
        Self {
            prev_query: String::new(),
            repeat: 0,
            group: String::new(),
            monitoring: false,
            focus_gd: false,
        }
    }
}

impl DaemonState {
    fn paths() -> (PathBuf, PathBuf) {
        let cfg = &crate::config::Config::global().paths;
        (cfg.state.clone(), cfg.lock.clone())
    }

    fn lock_file(path: &PathBuf) -> Result<std::fs::File> {
        let file = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .read(true)
            .truncate(true)
            .open(path)
            .map_err(|e| Error::Io(e))?;
        file.lock_exclusive().map_err(|e| Error::Io(e))?;
        Ok(file)
    }

    pub fn load() -> Result<Self> {
        let (state_path, lock_path) = Self::paths();
        let _lock = Self::lock_file(&lock_path)?;

        if !state_path.exists() {
            return Ok(DaemonState::default());
        }
        let content = std::fs::read_to_string(&state_path)
            .map_err(|e| Error::Io(e))?;
        if content.trim().is_empty() {
            return Ok(DaemonState::default());
        }
        let state: DaemonState = serde_json::from_str(&content)
            .unwrap_or_default();
        Ok(state)
    }

    pub fn save(&self) -> Result<()> {
        let (state_path, lock_path) = Self::paths();
        let _lock = Self::lock_file(&lock_path)?;
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| Error::Json(e))?;
        std::fs::write(&state_path, content)
            .map_err(|e| Error::Io(e))?;
        Ok(())
    }

    /// Atomically read, mutate, and write state under a single flock.
    pub fn update<F>(f: F) -> Result<Self>
    where
        F: FnOnce(&mut DaemonState),
    {
        let (state_path, lock_path) = Self::paths();
        let _lock = Self::lock_file(&lock_path)?;
        let mut state = if state_path.exists() {
            let content = std::fs::read_to_string(&state_path)
                .map_err(|e| Error::Io(e))?;
            if content.trim().is_empty() {
                DaemonState::default()
            } else {
                serde_json::from_str(&content).unwrap_or_default()
            }
        } else {
            DaemonState::default()
        };
        f(&mut state);
        let content = serde_json::to_string_pretty(&state)
            .map_err(|e| Error::Json(e))?;
        std::fs::write(&state_path, content)
            .map_err(|e| Error::Io(e))?;
        Ok(state)
    }

    pub fn advance(&mut self, query: &str, chain: &[String]) -> usize {
        if query == self.prev_query {
            self.repeat += 1;
        } else {
            self.prev_query = query.to_string();
            self.repeat = 0;
        }
        if !chain.is_empty() {
            self.group = chain[self.repeat % chain.len()].clone();
        }
        self.repeat
    }

    pub fn mark_done(&mut self, query: &str, group: &str) {
        self.prev_query = query.to_string();
        self.repeat = 0;
        self.group = group.to_string();
    }
}
