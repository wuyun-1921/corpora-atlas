pub mod ipc;
pub mod clipboard;
pub mod gd_clip;

use crate::error::Result;
use tokio::sync::mpsc;

#[derive(Default)]
pub struct Daemon;

struct ClipboardEvent {
    query: String,
    group: String,
    should_focus: bool,
}

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

        // Clipboard polling runs in a spawned task to avoid blocking IPC.
        // Results are sent via mpsc channel, keeping the select! loop responsive.
        let (clip_tx, mut clip_rx) = mpsc::channel::<ClipboardEvent>(8);
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(poll_interval);
            let mut prev_clip = String::new();
            loop {
                interval.tick().await;
                let state = crate::state::DaemonState::load().ok();
                let monitoring = state.as_ref().map(|s| s.monitoring).unwrap_or(false);
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
                let should_focus = state.map(|s| s.focus_gd).unwrap_or(false);
                let _ = clip_tx.send(ClipboardEvent { query, group, should_focus }).await;
            }
        });

        loop {
            tokio::select! {
                Some(ev) = clip_rx.recv() => {
                    clipboard::gd_lookup(&ev.query, &ev.group).await;
                    if ev.should_focus {
                        clipboard::focus_gd().await;
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
