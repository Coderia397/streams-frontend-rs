//! The recommendation engine: who is asking (account, profile, session) here, the parts by subject, all re-exported so
//! `services::ai_engine::fetch_feed_page` and the other paths used across the app keep working.
use serde::{Deserialize, Serialize};

use gloo_net::http::Request;

use crate::services::tmdb::MediaItem;

/// Base URL of the recommendation engine, from crate::config (see `engine_url` for the phone on the laptop's address).
fn base() -> &'static str {
    crate::config::engine_url()
}

thread_local! {
    /// Session number used while personalisation is off: it lives for this page load only and is never stored.
    static VOLATILE_SESSION: std::cell::RefCell<Option<String>> = const { std::cell::RefCell::new(None) };
}

/// A call to the engine that says who is asking: the account's token and the profile in use (the engine needs an account for everything but its health and feature switches, and it
/// limits what a profile may see). Without an account it goes out as it is.
pub(crate) async fn engine_get(url: &str) -> Result<gloo_net::http::Response, gloo_net::Error> {
    let mut req = Request::get(url);
    if let Some(session) = crate::services::account::ensure_fresh().await {
        req = req.header("Authorization", &format!("Bearer {}", session.access_token));
        if let Some(pid) = active_profile_id().filter(|p| crate::services::sync::is_uuid(p)) {
            req = req.header("X-Profile-Id", &pid);
        }
    }
    req.send().await
}

/// The profile in use as the browser remembers it.
fn active_profile_id() -> Option<String> {
    web_sys::window()?.local_storage().ok().flatten()?.get_item(crate::keys::ACTIVE_PROFILE).ok().flatten().filter(|p| !p.is_empty())
}

/// Where the recommendation engine lives. Used by the help center to check that it answers.
pub fn engine_base_url() -> &'static str {
    base()
}

fn get_or_create_session_id() -> String {
    if !crate::consent::personalization_allowed() {
        return VOLATILE_SESSION.with(|v| v.borrow_mut().get_or_insert_with(|| format!("sess_tmp_{}_{}", (js_sys::Math::random() * 1_000_000.0) as u32, js_sys::Date::now() as u64)).clone());
    }
    if let Some(storage) = web_sys::window().and_then(|w| w.local_storage().ok().flatten()) {
        if let Ok(Some(sid)) = storage.get_item(crate::keys::SESSION_ID) {
            if !sid.is_empty() {
                return sid;
            }
        }
        let new_sid = format!("sess_{}_{}", (js_sys::Math::random() * 1_000_000.0) as u32, js_sys::Date::now() as u64);
        let _ = storage.set_item(crate::keys::SESSION_ID, &new_sid);
        return new_sid;
    }
    format!("sess_anon_{}", js_sys::Date::now() as u64)
}

fn get_user_taste_affinity() -> Option<String> {
    let storage = web_sys::window().and_then(|w| w.local_storage().ok().flatten())?;
    storage.get_item(crate::keys::USER_AFFINITY).ok().flatten()
}

/// `&rating=PG` for the profile in use, so the engine only returns titles that profile may see (empty when there is no profile).
fn rating_query() -> String {
    crate::pages::settings::prefs::active_rating().map(|r| format!("&rating={r}")).unwrap_or_default()
}

mod feed;
mod menus;
mod titles;

pub use feed::*;
pub use menus::*;
pub use titles::*;
