pub async fn read_clipboard() -> String {
    let result = tokio::time::timeout(
        std::time::Duration::from_secs(2),
        tokio::process::Command::new("wl-paste")
            .arg("--no-newline")
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::null())
            .output(),
    )
    .await;
    match result {
        Ok(Ok(o)) if o.status.success() => String::from_utf8(o.stdout).unwrap_or_default(),
        _ => String::new(),
    }
}

pub fn prepare_query(raw: &str) -> Option<String> {
    let line = raw.split('\n').next()?.trim().to_string();
    let line = crate::strip::strip_query(&line);
    if line.is_empty() {
        return None;
    }
    let max_len = crate::config::Config::global().daemon.max_query_len;
    if line.len() > max_len {
        return None;
    }
    if !crate::lang::is_alpha(&line) {
        return None;
    }
    if line.starts_with("http://")
        || line.starts_with("https://")
        || line.starts_with("www.")
        || line.starts_with("ftp://")
    {
        return None;
    }
    if line.contains('/') {
        return None;
    }
    Some(line)
}

pub async fn notify(app_name: &str, message: &str, always: bool) {
    if !always {
        if let Ok(state) = crate::state::DaemonState::load() {
            if !state.monitoring {
                return;
            }
        }
    }
    let _ = tokio::process::Command::new("notify-send")
        .arg("-t")
        .arg("1000")
        .arg("-a")
        .arg("corpora-atlas")
        .arg(app_name)
        .arg(message)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn();
}

pub async fn gd_lookup(query: &str, group: &str) {
    let binary = &crate::config::Config::global().gd.binary;
    let mut cmd = tokio::process::Command::new(binary);
    if !group.is_empty() {
        cmd.arg("-g").arg(group);
    }
    cmd.arg(query);
    let _ = cmd
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn();
}

pub async fn focus_gd() {
    let app_id = &crate::config::Config::global().gd.window_app_id;
    let _ = tokio::process::Command::new("wlrctl")
        .arg("window")
        .arg("focus")
        .arg(app_id)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn();
}
