pub mod ipc;
pub mod clipboard;
pub mod cycle;

use crate::error::Result;

pub struct Daemon;

impl Daemon {
    pub fn new() -> Self {
        Self
    }

    pub async fn run(&self) -> Result<()> {
        let socket_path = &crate::config::Config::global().paths.socket;

        let _ = std::fs::remove_file(socket_path);

        let listener = tokio::net::UnixListener::bind(socket_path)
            .map_err(|e| crate::error::Error::Io(e))?;

        let poll_interval = std::time::Duration::from_secs_f64(
            crate::config::Config::global().daemon.poll_interval,
        );

        let mut interval = tokio::time::interval(poll_interval);
        let mut prev_clip = String::new();

        loop {
            tokio::select! {
                _ = interval.tick() => {
                    let state = crate::state::DaemonState::load().ok();
                    let monitoring = state.map(|s| s.monitoring).unwrap_or(false);
                    if !monitoring {
                        continue;
                    }
                    let clip = clipboard::read_clipboard().await;
                    if clip == prev_clip || clip.is_empty() {
                        continue;
                    }
                    prev_clip = clip;
                    let query = match clipboard::prepare_query(&prev_clip) {
                        Some(q) => q,
                        None => continue,
                    };
                    let (_script, chain) = crate::lang::triage(&query);
                    let group = chain.first().cloned().unwrap_or_default();
                    clipboard::gd_lookup(&query, &group).await;
                    if let Ok(s) = crate::state::DaemonState::load() {
                        if s.focus_gd {
                            clipboard::focus_gd().await;
                        }
                    }
                }
                accept = listener.accept() => {
                    match accept {
                        Ok((stream, _)) => {
                            tokio::spawn(async move {
                                let _ = ipc::handle_connection(stream).await;
                            });
                        }
                        Err(_) => {}
                    }
                }
            }
        }
    }
}
