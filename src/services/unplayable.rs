//! Trailers YouTube refused to play here (removed, private, not allowed in other sites' pages or in this country), kept in this browser so
//! the trailer picker skips them: the billboard and the title page move on to the next trailer at once, and every player skips them on the
//! next visit instead of showing the picture alone.
use leptos::prelude::*;
use std::cell::RefCell;

const KEY: &str = "streams_unplayable_trailers";
/// The newest this many are kept.
const KEEP: usize = 300;

thread_local! {
    static KNOWN: RefCell<Option<Vec<String>>> = const { RefCell::new(None) };
    static CHANGED: ArcRwSignal<u32> = ArcRwSignal::new(0);
}

fn storage() -> Option<web_sys::Storage> {
    web_sys::window()?.local_storage().ok().flatten()
}

fn with_known<T>(f: impl FnOnce(&mut Vec<String>) -> T) -> T {
    KNOWN.with(|k| {
        let mut k = k.borrow_mut();
        let list = k.get_or_insert_with(|| {
            storage()
                .and_then(|s| s.get_item(KEY).ok().flatten())
                .and_then(|raw| serde_json::from_str(&raw).ok())
                .unwrap_or_default()
        });
        f(list)
    })
}

/// True when YouTube refused this video here before.
pub fn is_unplayable(key: &str) -> bool {
    with_known(|list| list.iter().any(|k| k == key))
}

/// Notes a video YouTube refused for good (its codes 100, 101, 150, 153); the pickers that follow `track` choose again.
pub fn mark(key: &str) {
    let added = with_known(|list| {
        if list.iter().any(|k| k == key) {
            return false;
        }
        list.insert(0, key.to_string());
        list.truncate(KEEP);
        if let (Some(s), Ok(raw)) = (storage(), serde_json::to_string(list)) {
            let _ = s.set_item(KEY, &raw);
        }
        true
    });
    if added {
        CHANGED.with(|c| c.update(|n| *n += 1));
    }
}

/// Read inside a reactive closure or effect so that it runs again when a video is refused.
pub fn track() {
    CHANGED.with(|c| c.track());
}

/// YouTube's error codes that do not go away on a second try: not found (100), not allowed in other sites' pages (101, 150), and 153
/// (the page's address was not accepted). 2 (bad request) and 5 (the player failed) can pass.
pub fn is_lasting(code: u32) -> bool {
    matches!(code, 100 | 101 | 150 | 153)
}

/// The best trailer of a title that YouTube has not refused here (a player whose trailer was refused asks for the next one).
pub async fn next_trailer(id: u32, is_tv: bool) -> Option<(String, bool)> {
    let videos = crate::services::tmdb::fetch_videos(id, is_tv).await.ok()?;
    crate::services::tmdb::select_best_tmdb_trailer(&videos)
}
