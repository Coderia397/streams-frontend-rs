use leptos::prelude::*;
use std::collections::HashMap;
use crate::services::tmdb::MediaItem;

#[derive(Clone, Copy)]
pub struct UIStore {
    pub info_modal_open: RwSignal<bool>,
    pub info_modal_movie_id: RwSignal<Option<u32>>,
    pub info_modal_is_tv: RwSignal<bool>,
    pub active_popup_id: RwSignal<Option<String>>,
    pub ambient_color: RwSignal<(u8, u8, u8)>,
    pub preview_muted: RwSignal<bool>,
    pub modal_initial_time: RwSignal<f64>,
    pub modal_current_time: RwSignal<f64>,
    pub modal_closing_time: RwSignal<Option<(u32, f64)>>,
    pub hero_paused_by_modal: RwSignal<bool>,
    pub modal_video_key: RwSignal<Option<String>>,
    pub modal_is_teaser: RwSignal<bool>,
    /// One-shot seek signal: set when modal closes so hero TrailerPlayer jumps to exact timestamp.
    /// Cleared immediately after being consumed by the seek Effect.
    pub hero_resume_seek: RwSignal<Option<f64>>,
    /// The card of the New & Hot page whose trailer is playing now (one at a time): `movie-550`, `tv-1399`
    pub spotlight_playing: RwSignal<Option<String>>,
    /// True while the page is scrolling to a tab of New & Hot: no card starts its trailer on the way
    pub spotlight_hold: RwSignal<bool>,
}

/// The sound choice of the viewer: whatever they set last (kept in the browser across visits), not muted until they mute. The site never starts muted on its own account.
pub fn get_session_preview_muted(profile_id: Option<&str>, _is_kids: bool) -> bool {
    let read = |storage: Option<web_sys::Storage>| -> Option<bool> {
        let s = storage?;
        let from = |key: String| s.get_item(&key).ok().flatten().map(|v| v == "1" || v == "true");
        profile_id.and_then(|pid| from(crate::keys::preview_muted_for(pid))).or_else(|| from(crate::keys::PREVIEW_MUTED.to_string()))
    };
    let local = web_sys::window().and_then(|w| w.local_storage().ok().flatten());
    let session = web_sys::window().and_then(|w| w.session_storage().ok().flatten());
    read(local).or_else(|| read(session)).unwrap_or(false)
}

pub fn save_session_preview_muted(muted: bool, profile_id: Option<&str>) {
    let val = if muted { "1" } else { "0" };
    let Some(win) = web_sys::window() else { return };
    for storage in [win.local_storage().ok().flatten(), win.session_storage().ok().flatten()].into_iter().flatten() {
        let _ = storage.set_item(crate::keys::PREVIEW_MUTED, val);
        if let Some(pid) = profile_id {
            let _ = storage.set_item(&crate::keys::preview_muted_for(pid), val);
        }
    }
}

impl UIStore {
    pub fn set_preview_muted(&self, muted: bool) {
        self.preview_muted.set(muted);
        save_session_preview_muted(muted, None);
    }

    pub fn set_preview_muted_with_profile(&self, muted: bool, profile_id: Option<&str>) {
        self.preview_muted.set(muted);
        save_session_preview_muted(muted, profile_id);
    }
}

thread_local! {
    static KEPT: std::cell::RefCell<std::collections::HashMap<std::any::TypeId, Box<dyn std::any::Any>>> = std::cell::RefCell::new(std::collections::HashMap::new());
}

/// Provides an app store as context and keeps a copy of it here. The stores are made once, when the app starts, and live as long as
/// the page, so code that runs without a component around it (a click handler, a timer, a view closure that runs once more as its
/// component is taken down) still finds them: the context lookup alone panicked there (the phone's Who's watching, 2026-10-23).
pub(crate) fn provide_and_keep<T: Clone + Send + Sync + 'static>(store: T) {
    KEPT.with(|k| k.borrow_mut().insert(std::any::TypeId::of::<T>(), Box::new(store.clone())));
    provide_context(store);
}

/// A store from the context, or the copy kept when it was provided.
pub(crate) fn use_kept<T: Clone + Send + Sync + 'static>(name: &str) -> T {
    use_context::<T>()
        .or_else(|| KEPT.with(|k| k.borrow().get(&std::any::TypeId::of::<T>()).and_then(|b| b.downcast_ref::<T>()).cloned()))
        .unwrap_or_else(|| panic!("{name} not provided"))
}

pub fn provide_ui_store() {
    let initial_ambient = crate::utils::ambient::get_last_ambient_color().unwrap_or((16, 21, 25));
    let initial_muted = get_session_preview_muted(None, false);
    provide_and_keep(UIStore {
        info_modal_open: RwSignal::new(false),
        info_modal_movie_id: RwSignal::new(None),
        info_modal_is_tv: RwSignal::new(false),
        active_popup_id: RwSignal::new(None),
        ambient_color: RwSignal::new(initial_ambient),
        preview_muted: RwSignal::new(initial_muted),
        modal_initial_time: RwSignal::new(0.0),
        modal_current_time: RwSignal::new(0.0),
        modal_closing_time: RwSignal::new(None),
        hero_paused_by_modal: RwSignal::new(false),
        modal_video_key: RwSignal::new(None),
        modal_is_teaser: RwSignal::new(false),
        hero_resume_seek: RwSignal::new(None),
        spotlight_playing: RwSignal::new(None),
        spotlight_hold: RwSignal::new(false),
    });
}

pub fn use_ui_store() -> UIStore {
    use_kept::<UIStore>("UIStore")
}

#[derive(Clone, Copy)]
pub struct ProfileStore {
    pub active_profile_id: RwSignal<Option<String>>,
    pub profiles: RwSignal<Vec<crate::models::profile::Profile>>,
    pub is_authenticated: RwSignal<bool>,
    pub is_kids_mode: RwSignal<bool>,
    /// A line the profile screens show when the account refused a change (and the list went back to the account's).
    pub notice: RwSignal<Option<String>>,
}

/// The profiles are the account's only (owner, 2026-10-27, note 136): a browser without them has none, where it had a "Main" and a
/// "Kids" of its own that were never saved and that Who's watching offered to visitors, also on the Help Centre's host.
pub fn provide_profile_store() {
    let storage = web_sys::window().and_then(|w| w.local_storage().ok().flatten());
    // profiles kept by a browser without an account are those two, saved when one was picked on Who's watching: they go
    if let Some(s) = storage.as_ref().filter(|_| !crate::services::account::has_account()) {
        let _ = s.remove_item(crate::keys::PROFILES);
        let _ = s.remove_item(crate::keys::ACTIVE_PROFILE);
    }
    let mut initial_profiles = storage
        .as_ref()
        .and_then(|s| s.get_item(crate::keys::PROFILES).ok().flatten())
        .and_then(|raw| serde_json::from_str::<Vec<crate::models::profile::Profile>>(&raw).ok())
        .unwrap_or_default();
    crate::models::profile_rules::normalize(&mut initial_profiles);
    // the profile in use: the one this browser chose, if it still exists. Without one, a browser that holds an account shows Who's watching
    // (as Netflix does after signing in or reloading before a choice); a browser without an account uses its first profile, if it kept any.
    let saved = storage.as_ref().and_then(|s| s.get_item(crate::keys::ACTIVE_PROFILE).ok().flatten());
    let initial_profile = saved.filter(|id| initial_profiles.iter().any(|p| &p.id == id)).or_else(|| {
        if crate::services::account::has_account() { None } else { initial_profiles.first().map(|p| p.id.clone()) }
    });
    for p in initial_profiles.iter_mut() {
        p.avatar_url = p.avatar_url.take().map(|u| crate::data::avatars::fast_avatar(&u));
    }

    provide_and_keep(ProfileStore {
        active_profile_id: RwSignal::new(initial_profile),
        profiles: RwSignal::new(initial_profiles),
        is_authenticated: RwSignal::new(true),
        is_kids_mode: RwSignal::new(false),
        notice: RwSignal::new(None),
    });
}

/// Called after sign in or sign up. The first time on this browser it creates the main profile, named after the first name, and
/// the Kids profile; then it shows the Who's watching screen. Profiles are kept in this browser until the account stores them.
/// Removes everything in this browser that belongs to the signed in account: each profile's data (`scoped`), the profiles and the profile
/// in use, the profiles' and the account's settings, the taste session and the read notifications. What belongs to the browser stays
/// (language, cookie choices, Help Centre history, the device key, the app's own settings, a sealed "Remember me" session).
pub fn forget_account_data() {
    scoped::clear_all();
    let Some(storage) = web_sys::window().and_then(|w| w.local_storage().ok().flatten()) else { return };
    for k in [crate::keys::PROFILE_PREFS, crate::keys::ACCOUNT_PREFS, crate::keys::USER_AFFINITY, crate::keys::SESSION_ID, crate::keys::DATA_OWNER] {
        let _ = storage.remove_item(k);
    }
    let len = storage.length().unwrap_or(0);
    let read: Vec<String> = (0..len).filter_map(|i| storage.key(i).ok().flatten()).filter(|k| k.starts_with("streams-notifications-read:")).collect();
    for k in read {
        let _ = storage.remove_item(&k);
    }
}

/// True when this browser holds an account's data but no session for it: the session ended while the browser was closed, or a session
/// kept for this tab only ended with its tab. The app clears it at start (`lib.rs`).
pub fn orphaned_account_data() -> bool {
    let owner = web_sys::window().and_then(|w| w.local_storage().ok().flatten()).and_then(|s| s.get_item(crate::keys::DATA_OWNER).ok().flatten());
    owner.is_some() && !crate::services::account::has_account()
}

pub fn start_account(store: ProfileStore, session: &crate::services::account::Session) {
    let first_name = session.first_name.as_str();
    // another account's data never mixes with this one's
    let owner = web_sys::window().and_then(|w| w.local_storage().ok().flatten()).and_then(|s| s.get_item(crate::keys::DATA_OWNER).ok().flatten());
    if owner.as_deref().is_some_and(|o| o != session.user_id) {
        forget_account_data();
        store.profiles.set(Vec::new());
    }
    let storage = web_sys::window().and_then(|w| w.local_storage().ok().flatten());
    if let Some(s) = storage.as_ref() {
        let _ = s.set_item(crate::keys::DATA_OWNER, &session.user_id);
    }
    let has_saved = storage.as_ref().and_then(|s| s.get_item(crate::keys::PROFILES).ok().flatten()).is_some();
    if !has_saved {
        let name = if first_name.trim().is_empty() { "Me".to_string() } else { first_name.trim().chars().take(30).collect() };
        let profiles = vec![
            crate::models::profile::Profile { id: "main".to_string(), name, avatar_url: Some(crate::data::avatars::random_smile()), is_kids: false, is_default: Some(true), pin: None, sort_order: 0 },
            crate::models::profile::Profile { id: "kids".to_string(), name: crate::i18n::t("profile-default-kids"), avatar_url: None, is_kids: true, is_default: Some(false), pin: None, sort_order: 1 },
        ];
        if let (Some(s), Ok(json)) = (storage.as_ref(), serde_json::to_string(&profiles)) {
            let _ = s.set_item(crate::keys::PROFILES, &json);
        }
        store.profiles.set(profiles);
    }
    if let Some(s) = storage {
        let _ = s.remove_item(crate::keys::ACTIVE_PROFILE);
    }
    store.is_authenticated.set(true);
    store.active_profile_id.set(None);
    // the account's own profiles replace the ones made here
    crate::services::sync::refresh_profiles(store);
}

/// Makes a profile the one in use and loads the page afresh (the page Who's watching covered, or the profile's home after a switch), so everything that
/// belongs to a profile (feed, list, ratings, the Kids frame) is read again for it, as Netflix does when you pick a profile.
pub fn activate_profile(id: &str) {
    if let Some(window) = web_sys::window() {
        if let Some(storage) = window.local_storage().ok().flatten() {
            let _ = storage.set_item(crate::keys::ACTIVE_PROFILE, id);
        }
        if let Some(storage) = window.session_storage().ok().flatten() {
            for k in [crate::keys::KIDS_GATE, crate::keys::MANAGE_EDIT, crate::keys::GATE_RETURN] {
                let _ = storage.remove_item(k);
            }
        }
        // Who's watching came up over a page (an address opened or linked, a reload, the page left open when the browser asked again after
        // time away, the page Switch profiles was chosen on): the pick goes on to that page. A switch from one profile to another (the menu,
        // the Kids and Exit Kids buttons) starts on the new profile's home. The screen is up while no profile is in use; storage cannot tell,
        // as the screen saves its pick there before this runs.
        let from_gate = use_profile_store().active_profile_id.get_untracked().is_none();
        let path = window.location().pathname().unwrap_or_default();
        if from_gate && stays_after_pick(&path, home_of(id) == crate::config::routes::KIDS) {
            let search = window.location().search().unwrap_or_default();
            let _ = window.location().set_href(&format!("{path}{search}"));
            return;
        }
        // a title waiting in the address (a shared link, a reload while Who's watching was up) opens on the profile's home
        let pending = window.location().search().ok().and_then(|q| q.trim_start_matches('?').split('&').find(|kv| kv.starts_with("title=")).map(str::to_string));
        let home = home_of(id);
        let _ = window.location().set_href(&match pending {
            Some(title) => format!("{home}?{title}"),
            None => home.to_string(),
        });
    }
}

/// Brings up Who's watching over the page in use: the profile in use is put down, and the screen's pick comes back to this page
/// (`activate_profile`). Asked by a Kids profile (its Switch Profile, Exit Kids, time away on it), the screen is Netflix's Kids form: the
/// kids mark in the corner, no Add Profile and no Manage Profiles, and the same mark on the loading screen after the pick (owner's
/// recording, 2026-10-26, note 132). The form is kept for the tab, so a reload shows the same screen; a pick ends it.
pub fn ask_who_is_watching(store: ProfileStore) {
    let from_kids = store.active_profile_id.get_untracked().is_some_and(|id| store.profiles.with_untracked(|l| l.iter().any(|p| p.id == id && p.is_kids)));
    if let Some(s) = web_sys::window().and_then(|w| w.session_storage().ok().flatten()) {
        let _ = if from_kids { s.set_item(crate::keys::KIDS_GATE, "1") } else { s.remove_item(crate::keys::KIDS_GATE) };
    }
    if let Some(s) = web_sys::window().and_then(|w| w.local_storage().ok().flatten()) {
        let _ = s.remove_item(crate::keys::ACTIVE_PROFILE);
    }
    store.active_profile_id.set(None);
}

fn session() -> Option<web_sys::Storage> {
    web_sys::window().and_then(|w| w.session_storage().ok().flatten())
}

/// Manage Profiles opened this profile's settings (after its PIN, if it has one): while no profile is in use, those pages show without
/// Who's watching over them, as Netflix's "Manage profile and preferences" opened from its Manage Profiles page (note 132).
pub fn open_from_manage(id: &str) {
    if let Some(s) = session() { let _ = s.set_item(crate::keys::MANAGE_EDIT, id); }
}

/// The profile whose settings Manage Profiles opened (`open_from_manage`).
pub fn manage_edit() -> Option<String> {
    session().and_then(|s| s.get_item(crate::keys::MANAGE_EDIT).ok().flatten())
}

/// Whether `path` is a page of its own while no profile is in use: Manage Profiles, and the settings of the profile it opened.
pub fn shown_without_profile(path: &str) -> bool {
    let path = path.trim_end_matches('/');
    let edited = manage_edit().is_some_and(|id| {
        let root = format!("/settings/profile/{id}");
        path == root || path.starts_with(&format!("{root}/"))
    });
    path == "/profiles/manage" || edited
}

/// Remembers the page Who's watching was over when it sent the person to Manage Profiles, for Done to come back to.
pub fn remember_gate_page(path: &str) {
    if let Some(s) = session() { let _ = s.set_item(crate::keys::GATE_RETURN, path); }
}

/// The page Who's watching was over (`remember_gate_page`), else Home.
pub fn gate_page() -> String {
    session().and_then(|s| s.get_item(crate::keys::GATE_RETURN).ok().flatten()).filter(|p| p.starts_with('/') && !p.starts_with("//")).unwrap_or_else(|| crate::config::routes::BROWSE.to_string())
}

/// Whether Who's watching was asked by a Kids profile (`ask_who_is_watching`).
pub fn kids_gate() -> bool {
    web_sys::window().and_then(|w| w.session_storage().ok().flatten()).and_then(|s| s.get_item(crate::keys::KIDS_GATE).ok().flatten()).is_some()
}

/// Whether a pick on Who's watching goes on to the page under it, as Netflix keeps its address. The two homes lead to the picked profile's
/// own home, and the sign in, email link and profile pages to Home. A Kids profile goes on only where the page checks it (the player checks
/// the age rating, a genre page shows a Kids profile's titles); anywhere else it starts on `/kids`.
fn stays_after_pick(path: &str, kids: bool) -> bool {
    use crate::config::routes;
    let path = path.trim_end_matches('/');
    if matches!(path, "" | routes::BROWSE | routes::KIDS) || [routes::LOGIN, "/signup", "/auth", "/profiles"].iter().any(|p| path.starts_with(p)) {
        return false;
    }
    !kids || path.starts_with("/watch/") || path.starts_with("/browse/genre/")
}

/// Where a profile starts: a Kids profile at `/kids`, every other one at `/browse`.
pub fn home_of(id: &str) -> &'static str {
    let kids = web_sys::window()
        .and_then(|w| w.local_storage().ok().flatten())
        .and_then(|s| s.get_item(crate::keys::PROFILES).ok().flatten())
        .and_then(|raw| serde_json::from_str::<Vec<crate::models::profile::Profile>>(&raw).ok())
        .is_some_and(|list| list.iter().any(|p| p.id == id && p.is_kids));
    if kids { crate::config::routes::KIDS } else { crate::config::routes::BROWSE }
}

pub fn use_profile_store() -> ProfileStore {
    use_kept::<ProfileStore>("ProfileStore")
}

#[derive(Clone, Debug)]
pub struct LibraryEntry {
    pub media: MediaItem,
    pub added_at: u64,
}

#[derive(Clone, Copy)]
pub struct LibraryStore {
    pub my_list: RwSignal<HashMap<u32, LibraryEntry>>,
}

/// What is saved of My List: the title as the pages show it and when it was added.
#[derive(serde::Serialize, serde::Deserialize)]
struct SavedEntry {
    media: MediaItem,
    added_at: u64,
}

const MY_LIST_KEY: &str = "streams_my_list_v1";

fn read_my_list() -> HashMap<u32, LibraryEntry> {
    web_sys::window()
        .and_then(|w| w.local_storage().ok().flatten())
        .and_then(|s| s.get_item(&scoped::key(MY_LIST_KEY)).ok().flatten())
        .and_then(|raw| serde_json::from_str::<Vec<SavedEntry>>(&raw).ok())
        .map(|v| v.into_iter().map(|e| (e.media.id, LibraryEntry { media: e.media, added_at: e.added_at })).collect())
        .unwrap_or_default()
}

fn write_my_list(map: &HashMap<u32, LibraryEntry>) {
    let list: Vec<SavedEntry> = map.values().map(|e| SavedEntry { media: e.media.clone(), added_at: e.added_at }).collect();
    if let (Some(storage), Ok(raw)) = (web_sys::window().and_then(|w| w.local_storage().ok().flatten()), serde_json::to_string(&list)) {
        let _ = storage.set_item(&scoped::key(MY_LIST_KEY), &raw);
    }
}

pub fn provide_library_store() {
    let my_list = RwSignal::new(read_my_list());
    // every change is kept in this browser and sent to the account a few seconds later
    Effect::new(move |first: Option<()>| {
        let snapshot = my_list.get();
        if first.is_some() {
            write_my_list(&snapshot);
            crate::services::sync::schedule_push();
        }
    });
    provide_and_keep(LibraryStore { my_list });
}

impl LibraryStore {
    /// Reads the list of the profile now in use (after the profile changed).
    pub fn reload(&self) {
        self.my_list.set(read_my_list());
    }
}

pub fn use_library_store() -> LibraryStore {
    use_kept::<LibraryStore>("LibraryStore")
}

pub mod ratings;
pub mod scoped;
pub mod trailers;
pub mod watch_store;
pub use watch_store::{provide_watch_store, use_watch_store, WatchRecord, WatchStore};

#[cfg(test)]
mod tests {
    use super::stays_after_pick;

    #[test]
    fn a_pick_goes_on_to_the_page_under_who_is_watching() {
        for p in ["/browse/genre/83", "/title/tv-1399", "/search", "/browse/my-list", "/latest", "/settings/account", "/help/contact", "/watch/tv/1399"] {
            assert!(stays_after_pick(p, false), "{p}");
        }
        for p in ["/", "", "/browse", "/browse/", "/kids", "/login", "/signup", "/auth/confirm", "/profiles/manage"] {
            assert!(!stays_after_pick(p, false), "{p}");
        }
        // a Kids profile goes on only where the page checks it
        for (p, stays) in [("/watch/movie/12", true), ("/browse/genre/83", true), ("/title/tv-1399", false), ("/search", false), ("/browse/my-list", false)] {
            assert_eq!(stays_after_pick(p, true), stays, "{p}");
        }
    }
}
