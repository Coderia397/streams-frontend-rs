//! Storage keys that belong to one profile. What a profile holds (viewing progress, My List, ratings, what was last sent to the account) is kept in this browser under the key of the
//! store plus the id of the profile in use, so two profiles of one account (or two accounts on one computer) never see each other's data. Signing out removes all of it.

/// The id of the profile in use as the browser remembers it (`none` before one is chosen).
pub fn active() -> String {
    web_sys::window()
        .and_then(|w| w.local_storage().ok().flatten())
        .and_then(|s| s.get_item(crate::keys::ACTIVE_PROFILE).ok().flatten())
        .filter(|p| !p.is_empty())
        .unwrap_or_else(|| "none".to_string())
}

/// `base` for the profile in use.
pub fn key(base: &str) -> String {
    format!("{base}:{}", active())
}

/// The bases of the keys above, so that signing out can remove every profile's copy.
pub const BASES: [&str; 7] = [
    "streams_watch_progress_v1",
    "streams_episode_progress_v1",
    "streams_my_list_v1",
    "streams_title_ratings_v1",
    "streams_title_ratings_ts_v1",
    "streams_sync_snapshot",
    "streams_sync_cursor",
];

/// Removes this browser's copy of every profile's data (signing out of a shared computer).
pub fn clear_all() {
    let Some(storage) = web_sys::window().and_then(|w| w.local_storage().ok().flatten()) else { return };
    let len = storage.length().unwrap_or(0);
    let mut doomed = Vec::new();
    for i in 0..len {
        if let Ok(Some(k)) = storage.key(i) {
            if BASES.iter().any(|b| k == *b || k.starts_with(&format!("{b}:"))) || k == crate::keys::PROFILES || k == crate::keys::ACTIVE_PROFILE {
                doomed.push(k);
            }
        }
    }
    for k in doomed {
        let _ = storage.remove_item(&k);
    }
}
