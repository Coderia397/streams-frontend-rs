//! Storage keys and DOM ids shared between Rust and the TypeScript bridge. The values are frozen: existing browsers hold
//! data under these names, so renaming one needs a migration that copies the old value to the new key first (see legacy_keys.rs
//! for the move from the first product name).

pub const ACTIVE_PROFILE: &str = "streams_active_profile";
/// When this browser was last used (milliseconds): a long time away brings back Who's watching (services/profile_gate.rs).
pub const LAST_SEEN: &str = "streams_last_seen";
/// Who's watching was asked by a Kids profile (this tab only): the Kids form of the screen (store::ask_who_is_watching)
pub const KIDS_GATE: &str = "streams_kids_gate";
pub const PROFILES: &str = "streams_profiles";
pub const PREVIEW_MUTED: &str = "streams_preview_muted";
pub const SESSION_ID: &str = "streams_session_id";
pub const USER_AFFINITY: &str = "streams_user_affinity";
pub const TITLE_RATINGS: &str = "streams_title_ratings_v1";
pub const LANGUAGE: &str = "streams_language";
pub const NOTIFICATIONS_READ: &str = "streams-notifications-read:default";
pub const SESSION: &str = "streams_session";
pub const CONSENT: &str = "streams_consent";
pub const CONSENT_DISMISSED: &str = "streams_consent_dismissed";
pub const HELP_RECENT: &str = "streams_help_recent";
pub const HELP_VIEWED: &str = "streams_help_viewed";
pub const HELP_VOTES: &str = "streams_help_votes";
pub const PROFILE_PREFS: &str = "streams_profile_prefs";
pub const APP_PREFS: &str = "streams_app_prefs";
pub const ACCOUNT_PREFS: &str = "streams_account_prefs";
/// The account (user id) whose data this browser holds, so a session that ended while the browser was closed, or a sign in to
/// another account, clears it (see store::forget_account_data).
pub const DATA_OWNER: &str = "streams_data_owner";
/// When the app last started in this browser (milliseconds); index.html reads it to decide whether to show the start animation.
/// Random id this browser keeps so an account can recognise it again (see services/device.rs).
pub const DEVICE_KEY: &str = "streams_device_key";
/// Country of the connection (two letters) as the edge reported it at the last registration, and the rating system the database resolved
/// for this session ("uk" or "us"). Both are only caches for showing labels without waiting for the network.
pub const COUNTRY: &str = "streams_country";
pub const RATING_SYSTEM_AUTO: &str = "streams_rating_system_auto";
pub const BOOT_SEEN: &str = "streams_boot_seen";

pub const VIDEO_ELEMENT_ID: &str = "streams-video";
pub const PLAYER_CONTAINER_ID: &str = "streams-main-player";

pub fn preview_muted_for(profile_id: &str) -> String {
    format!("{PREVIEW_MUTED}_{profile_id}")
}
