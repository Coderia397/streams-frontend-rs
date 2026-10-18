//! What the settings screens save. Per profile: languages, subtitle look, autoplay, age limit, blocked titles. Per device: data use
//! and notifications. Everything is kept in this browser first; the account tables take over after the database setup is run.
use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Subtitle look, the same six choices Netflix offers on phones and tablets.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct SubtitleStyle {
    /// 0 small, 1 medium, 2 large
    pub size: u8,
    pub color: String,
    pub edge: String,
    pub edge_color: String,
    pub background: String,
    pub background_clear: bool,
    pub window: String,
    pub window_clear: bool,
}

impl Default for SubtitleStyle {
    fn default() -> Self {
        Self {
            size: 1,
            color: "white".into(),
            edge: "shadow".into(),
            edge_color: "black".into(),
            background: "none".into(),
            background_clear: false,
            window: "none".into(),
            window_clear: false,
        }
    }
}

/// Settings of one profile. The field names follow the columns of public.profile_settings (autoplay_next, autoplay_previews,
/// audio_language, subtitle_language, subtitle_style, ui_language) and profile_settings.age_rating, so syncing them is a copy, not a
/// translation. Blocked titles have no column yet.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ProfilePrefs {
    pub autoplay_next: bool,
    pub autoplay_previews: bool,
    /// "email", "push" and "<channel>.<topic>" switches; a missing key means on.
    pub notifications: HashMap<String, bool>,
    /// "auto", "low", "medium" or "high" (profile_settings.data_usage).
    pub data_usage: String,
    /// Name in Netflix style games (profile_settings.game_handle).
    pub game_handle: Option<String>,
    /// "woman", "man" or "none" (Edit Profile).
    pub gender: Option<String>,
    pub audio_language: Option<String>,
    pub subtitle_language: Option<String>,
    pub ui_language: Option<String>,
    /// Highest British rating shown (key from data::settings::RATINGS); None means the default for the kind of profile.
    pub max_rating: Option<String>,
    pub blocked_titles: Vec<String>,
    pub subtitle_style: SubtitleStyle,
    /// When these settings last changed on this device, in milliseconds (0 never): the newer side wins when the account's copy is read.
    pub updated_at: u64,
}

impl Default for ProfilePrefs {
    fn default() -> Self {
        Self { autoplay_next: true, autoplay_previews: true, notifications: HashMap::new(), data_usage: "auto".into(), game_handle: None, gender: None, audio_language: None, subtitle_language: None, ui_language: None, max_rating: None, blocked_titles: Vec::new(), subtitle_style: SubtitleStyle::default(), updated_at: 0 }
    }
}

impl ProfilePrefs {
    /// A notification switch; anything never set counts as on.
    pub fn notify(&self, key: &str) -> bool {
        self.notifications.get(key).copied().unwrap_or(true)
    }

    /// The rating limit in force: the saved one if it is allowed for this kind of profile, otherwise the default for it.
    pub fn rating_for(&self, kids: bool) -> &'static str {
        use crate::data::settings::{rating, DEFAULT_KIDS_RATING, DEFAULT_RATING};
        let default = if kids { DEFAULT_KIDS_RATING } else { DEFAULT_RATING };
        let level = rating(self.max_rating.as_deref().unwrap_or(default));
        if kids && !level.kids { rating(DEFAULT_KIDS_RATING).key } else { level.key }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct AppPrefs {
    /// "auto", "save" or "max"
    pub data_usage: String,
    pub notifications: bool,
}

impl Default for AppPrefs {
    fn default() -> Self {
        Self { data_usage: "auto".into(), notifications: true }
    }
}

/// Switches of the whole account (account_details.test_participation and profile_transfer).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct AccountPrefs {
    pub test_participation: bool,
    pub profile_transfer: bool,
    /// "auto" (follow the device), "uk" or "us": which age rating labels are read (account_details.rating_system)
    pub rating_system: String,
}

impl Default for AccountPrefs {
    fn default() -> Self {
        Self { test_participation: true, profile_transfer: true, rating_system: "auto".into() }
    }
}

#[derive(Clone, Copy)]
pub struct Prefs {
    profiles: RwSignal<HashMap<String, ProfilePrefs>>,
    pub app: RwSignal<AppPrefs>,
    pub account: RwSignal<AccountPrefs>,
    /// Unix seconds until which the password check counts. Kept in memory only, a reload asks again.
    pub verified_until: RwSignal<u64>,
}

fn read<T: for<'de> Deserialize<'de>>(key: &str) -> Option<T> {
    let raw = web_sys::window()?.local_storage().ok().flatten()?.get_item(key).ok().flatten()?;
    serde_json::from_str(&raw).ok()
}

fn write<T: Serialize>(key: &str, value: &T) {
    if let (Some(storage), Ok(json)) = (web_sys::window().and_then(|w| w.local_storage().ok().flatten()), serde_json::to_string(value)) {
        let _ = storage.set_item(key, &json);
    }
}

pub fn now_secs() -> u64 {
    (js_sys::Date::now() / 1000.0) as u64
}

pub fn provide_prefs() {
    let mut saved: HashMap<String, ProfilePrefs> = read(crate::keys::PROFILE_PREFS).unwrap_or_default();
    for p in saved.values_mut() {
        if p.updated_at == 0 && *p != ProfilePrefs::default() { p.updated_at = js_sys::Date::now() as u64; }
    }
    let prefs = Prefs {
        profiles: RwSignal::new(saved),
        app: RwSignal::new(read(crate::keys::APP_PREFS).unwrap_or_default()),
        account: RwSignal::new(read(crate::keys::ACCOUNT_PREFS).unwrap_or_default()),
        verified_until: RwSignal::new(0),
    };
    Effect::new(move |_| prefs.profiles.with(|p| write(crate::keys::PROFILE_PREFS, p)));
    Effect::new(move |_| prefs.app.with(|a| write(crate::keys::APP_PREFS, a)));
    Effect::new(move |_| prefs.account.with(|a| write(crate::keys::ACCOUNT_PREFS, a)));
    provide_context(prefs);
}

pub fn use_prefs() -> Prefs {
    use_context::<Prefs>().expect("Prefs not provided")
}

impl Prefs {
    pub fn profile(&self, id: &str) -> ProfilePrefs {
        self.profiles.with(|m| m.get(id).cloned().unwrap_or_default())
    }

    pub fn update_profile(&self, id: &str, change: impl FnOnce(&mut ProfilePrefs)) {
        self.profiles.update(|m| {
            let p = m.entry(id.to_string()).or_default();
            change(p);
            p.updated_at = js_sys::Date::now() as u64;
        });
        // the account is told a few seconds later
        crate::services::sync::settings_changed(id);
    }

    /// Takes the account's copy of a profile's settings (no time change, no push back).
    pub fn apply_remote(&self, id: &str, prefs: ProfilePrefs) {
        self.profiles.update(|m| { m.insert(id.to_string(), prefs); });
    }

    pub fn forget_profile(&self, id: &str) {
        self.profiles.update(|m| {
            m.remove(id);
        });
    }

    pub fn forget_all(&self) {
        self.profiles.set(HashMap::new());
        self.app.set(AppPrefs::default());
        self.account.set(AccountPrefs::default());
    }

    /// The rating labels this person reads: their choice; on auto the answer the database gave for this session's device, else the country of
    /// the connection, else British. Reactive: reading it inside a view follows the setting.
    pub fn rating_system(&self) -> crate::data::ratings::RatingSystem {
        use crate::data::ratings::{system_for_country, RatingSystem};
        let choice = self.account.with(|a| a.rating_system.clone());
        RatingSystem::from_key(&choice)
            .or_else(|| crate::services::device::cached_rating_system().and_then(|k| RatingSystem::from_key(&k)))
            .or_else(|| crate::services::device::cached_country().map(|c| system_for_country(&c)))
            .unwrap_or(RatingSystem::Uk)
    }

    pub fn is_verified(&self) -> bool {
        self.verified_until.get_untracked() > now_secs()
    }

    pub fn mark_verified(&self) {
        self.verified_until.set(now_secs() + crate::config::RECENT_SIGN_IN_SECONDS);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn old_saved_data_still_loads() {
        let p: ProfilePrefs = serde_json::from_str(r#"{"autoplay_next":false}"#).unwrap();
        assert!(!p.autoplay_next);
        assert!(p.autoplay_previews);
        assert_eq!(p.subtitle_style, SubtitleStyle::default());
    }

    #[test]
    fn the_default_kids_profile_is_a_kids_profile() {
        let kids: Vec<_> = crate::store::default_profiles().into_iter().filter(|p| p.is_kids).collect();
        assert_eq!(kids.len(), 1);
        assert_eq!(kids[0].id, "p2");
    }

    #[test]
    fn a_kids_profile_never_gets_an_adult_limit() {
        let mut p = ProfilePrefs::default();
        assert_eq!(p.rating_for(false), "18");
        assert_eq!(p.rating_for(true), "PG");
        p.max_rating = Some("18".into());
        assert_eq!(p.rating_for(true), "PG");
        p.max_rating = Some("15".into());
        assert_eq!(p.rating_for(false), "15");
    }
}

/// True when the profile in use is a Kids profile, read straight from the browser store (not reactive; the page reloads when the profile changes).
pub fn active_is_kids() -> bool {
    let Some(storage) = web_sys::window().and_then(|w| w.local_storage().ok().flatten()) else { return false };
    let active = storage.get_item(crate::keys::ACTIVE_PROFILE).ok().flatten();
    let profiles: Vec<crate::models::profile::Profile> = read(crate::keys::PROFILES).unwrap_or_else(crate::store::default_profiles);
    profiles.iter().find(|p| Some(&p.id) == active.as_ref()).is_some_and(|p| p.is_kids)
}

/// The age rating limit of the profile in use (U, PG, 12, 15 or 18), read straight from the browser store, for requests to the engine.
/// Not reactive: it is read when a request is made. None when there is no profile yet.
pub fn active_rating() -> Option<&'static str> {
    let storage = web_sys::window().and_then(|w| w.local_storage().ok().flatten())?;
    let active = storage.get_item(crate::keys::ACTIVE_PROFILE).ok().flatten();
    // a browser that has saved no profiles still has the default main and Kids profiles, and the Kids one must be limited
    let profiles: Vec<crate::models::profile::Profile> = read(crate::keys::PROFILES).unwrap_or_else(crate::store::default_profiles);
    let profile = profiles.iter().find(|p| Some(&p.id) == active.as_ref()).or_else(|| profiles.first())?;
    let prefs: HashMap<String, ProfilePrefs> = read(crate::keys::PROFILE_PREFS).unwrap_or_default();
    Some(prefs.get(&profile.id).cloned().unwrap_or_default().rating_for(profile.is_kids))
}
