use crate::daemon::clipboard;
use crate::error::{Error, Result};

pub async fn handle_connection(mut stream: tokio::net::UnixStream) -> Result<()> {
    use tokio::io::AsyncReadExt;
    use tokio::io::AsyncWriteExt;

    let mut buf = vec![0u8; 4096];
    let n = stream
        .read(&mut buf)
        .await
        .map_err(|e| Error::Io(e))?;
    let raw = String::from_utf8_lossy(&buf[..n]).trim().to_string();
    let req: serde_json::Value = match serde_json::from_str(&raw) {
        Ok(v) => v,
        Err(_) => {
            let _ = stream.write_all(b"{\"error\":\"invalid json\"}\n").await;
            return Ok(());
        }
    };

    let action = req["action"].as_str().unwrap_or("");
    match action {
        "cycle" => {
            let clip = req.get("clip").and_then(|c| c.as_str()).unwrap_or("");
            crate::daemon::gd_clip::cmd_next(clip).await;
            let _ = stream.write_all(b"{\"status\":\"ok\"}\n").await;
        }
        "toggle" => {
            let mut state = crate::state::DaemonState::load()?;
            state.monitoring = !state.monitoring;
            state.save()?;
            let label = if state.monitoring { "enabled" } else { "disabled" };
            clipboard::notify("corpora-atlas", &format!(" monitoring {label}"), true).await;
            let resp = serde_json::json!({"status": "ok", "monitoring": state.monitoring});
            let _ = stream.write_all(format!("{resp}\n").as_bytes()).await;
        }
        "toggle_focus" => {
            let mut state = crate::state::DaemonState::load()?;
            state.focus_gd = !state.focus_gd;
            state.save()?;
            let label = if state.focus_gd { "enabled" } else { "disabled" };
            clipboard::notify("corpora-atlas", &format!(" GD auto-focus {label}"), true).await;
            let resp = serde_json::json!({"status": "ok", "focus_gd": state.focus_gd});
            let _ = stream.write_all(format!("{resp}\n").as_bytes()).await;
        }
        _ => {
            let _ = stream.write_all(b"{\"error\":\"unknown\"}\n").await;
        }
    }
    Ok(())
}

pub async fn send_daemon(action: &str, extra: Option<(&str, &str)>) -> Result<serde_json::Value> {
    use tokio::io::AsyncReadExt;
    use tokio::io::AsyncWriteExt;

    let socket_path = &crate::config::Config::global().paths.socket;
    if !socket_path.exists() {
        return Err(Error::Other("daemon not running".into()));
    }

    let stream = tokio::net::UnixStream::connect(socket_path)
        .await
        .map_err(|e| Error::Io(e))?;
    let (mut reader, mut writer) = stream.into_split();

    let mut req = serde_json::json!({"action": action});
    if let Some((k, v)) = extra {
        req[k] = serde_json::Value::String(v.to_string());
    }
    writer
        .write_all(format!("{req}\n").as_bytes())
        .await
        .map_err(|e| Error::Io(e))?;

    let mut buf = vec![0u8; 4096];
    let n = reader
        .read(&mut buf)
        .await
        .map_err(|e| Error::Io(e))?;
    let resp: serde_json::Value = serde_json::from_slice(&buf[..n])
        .map_err(|e| Error::Json(e))?;
    Ok(resp)
}
