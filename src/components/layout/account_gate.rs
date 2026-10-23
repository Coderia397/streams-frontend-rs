//! The account gate: everything in the app needs an account, except the pages a visitor may see (the landing page, a title's own page, signing in and up, help, the legal pages). A visitor who opens any other
//! address is taken to the sign in page, which sends them back where they were going after they sign in. The engine can switch the gate off for a while (`accounts_required: false` in
//! `features.json`, read by `/api/features`); an engine that cannot be reached counts as "required".

use leptos::prelude::*;
use leptos_router::hooks::{use_location, use_navigate};

/// Pages a visitor may open without an account.
pub fn is_open_to_visitors(path: &str) -> bool {
    const OPEN: [&str; 15] = ["/title", "/login", "/signup", "/forgot", "/auth", "/help", "/privacy", "/terms", "/cookies", "/dmca", "/credits", "/disclaimer", "/contact", "/welcome", "/reads"];
    path == "/" || path.is_empty() || OPEN.iter().any(|p| path == *p || path.starts_with(&format!("{p}/")))
}

#[component]
pub fn AccountGate() -> impl IntoView {
    let location = use_location();
    let navigate = use_navigate();
    let features = LocalResource::new(|| async { crate::services::ai_engine::fetch_features().await });
    Effect::new(move |_| {
        let path = location.pathname.get();
        if is_open_to_visitors(&path) || crate::services::account::has_account() {
            return;
        }
        // the engine answers first (an engine that cannot be reached answers "required"); the switch is off only when it has said so
        let Some(flags) = features.get() else { return };
        if flags.get("accounts_required") == Some(&false) {
            return;
        }
        let search = location.search.get();
        let search = if search.is_empty() || search.starts_with('?') { search } else { format!("?{search}") };
        let back = js_sys::encode_uri_component(&format!("{path}{search}")).as_string().unwrap_or_default();
        navigate(&format!("/login?next={back}"), leptos_router::NavigateOptions { replace: true, ..Default::default() });
    });
}

/// Where to go after signing in: the address the gate took the person from (only an address of this site), otherwise Home.
pub fn after_sign_in() -> String {
    let search = web_sys::window().and_then(|w| w.location().search().ok()).unwrap_or_default();
    let next = search.trim_start_matches('?').split('&').find_map(|kv| kv.strip_prefix("next=")).map(|v| js_sys::decode_uri_component(v).ok().and_then(|s| s.as_string()).unwrap_or_default());
    match next {
        Some(n) if n.starts_with('/') && !n.starts_with("//") && !is_open_to_visitors(n.split('?').next().unwrap_or("")) => n,
        _ => crate::config::routes::BROWSE.to_string(),
    }
}
