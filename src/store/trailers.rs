//! The trailers a person has watched, newest first, kept per profile in this browser: the "Trailers You've Watched" shelf of the My page.
const KEY: &str = "streams_trailers_watched_v1";
/// The shelf keeps this many.
const KEEP: usize = 40;

/// (media id, is a series), newest first.
pub fn all() -> Vec<(u32, bool)> {
    web_sys::window()
        .and_then(|w| w.local_storage().ok().flatten())
        .and_then(|s| s.get_item(&crate::store::scoped::key(KEY)).ok().flatten())
        .and_then(|raw| serde_json::from_str(&raw).ok())
        .unwrap_or_default()
}

/// Notes that the trailer of a title started playing: it goes to the front, once.
pub fn mark(id: u32, is_tv: bool) {
    let mut list = all();
    list.retain(|e| *e != (id, is_tv));
    list.insert(0, (id, is_tv));
    list.truncate(KEEP);
    if let (Some(storage), Ok(raw)) = (web_sys::window().and_then(|w| w.local_storage().ok().flatten()), serde_json::to_string(&list)) {
        let _ = storage.set_item(&crate::store::scoped::key(KEY), &raw);
    }
    crate::services::sync::touch_settings();
}

/// Takes the account's copy of the shelf.
pub fn replace(items: &[(u32, bool)]) {
    let list: Vec<(u32, bool)> = items.iter().take(KEEP).cloned().collect();
    if let (Some(storage), Ok(raw)) = (web_sys::window().and_then(|w| w.local_storage().ok().flatten()), serde_json::to_string(&list)) {
        let _ = storage.set_item(&crate::store::scoped::key(KEY), &raw);
    }
}
