use crate::daemon::clipboard;
use crate::state::DaemonState;

pub async fn cmd_next(clip_override: &str) {
    let clip = if clip_override.is_empty() {
        clipboard::read_clipboard().await
    } else {
        clip_override.to_string()
    };

    let query = match clipboard::prepare_query(&clip) {
        Some(q) => q,
        None => return,
    };

    let (script, chain) = crate::lang::triage(&query);

    if script == crate::lang::Script::Latin {
        let gd_backend = crate::backends::gd::GdBackend;
        let gd_group = gd_backend.get_current_group().await.unwrap_or_default();
        clipboard::gd_lookup(&query, &gd_group).await;
        clipboard::focus_gd().await;
        return;
    }

    let mut state = DaemonState::load().unwrap_or_default();
    let current_group = state.group.clone();
    let gd_backend = crate::backends::gd::GdBackend;
    let gd_group = gd_backend.get_current_group().await.unwrap_or_default();

    let group = if !gd_group.is_empty() && gd_group != current_group {
        if query != state.prev_query {
            state.prev_query = query.clone();
            state.repeat = 0;
            state.group = chain.first().cloned().unwrap_or_default();
            state.group.clone()
        } else if chain.contains(&gd_group) {
            let idx = chain.iter().position(|g| *g == gd_group).unwrap_or(0);
            state.repeat = idx + 1;
            state.group = chain[(idx + 1) % chain.len()].clone();
            state.group.clone()
        } else {
            state.group = gd_group.clone();
            state.group.clone()
        }
    } else {
        state.advance(&query, &chain);
        state.group.clone()
    };

    let _ = state.save();
    clipboard::gd_lookup(&query, &group).await;
    clipboard::focus_gd().await;
}
