//! The account gate: one rule for every address. Each address belongs to visitors (signing in, signing up, the forgotten password page),
//! to everyone (the landing page, a title's own page, help, the policies, the links in account emails), or to members (the rest of the
//! app). The gate follows the address and the signed in signal together (`account::signed_in`, which also changes when another tab signs
//! in or out): a member on a visitors' page goes where they were heading, or Home, as Netflix sends a signed in member away from its sign
//! in page; a visitor on a members' page goes to sign in, which sends them back after they sign in. The engine can switch the members'
//! side off for a while (`accounts_required: false` in `features.json`, read by `/api/features`); an engine that cannot be reached counts
//! as "required".

use leptos::prelude::*;
use leptos_router::hooks::{use_location, use_navigate};

/// Who an address is for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Access {
    /// signing in and up: a member is sent on
    Visitors,
    /// anyone; the root address chooses between the landing page and Home itself
    Everyone,
    /// the app
    Members,
}

/// Who an address is for (its path, without the query).
pub fn access(path: &str) -> Access {
    const VISITORS: [&str; 3] = ["/login", "/signup", "/forgot"];
    // `/auth` holds the links in account emails (confirm an address, set a new password): they work whether or not the browser is signed in
    const EVERYONE: [&str; 12] = ["/title", "/auth", "/help", "/privacy", "/terms", "/cookies", "/dmca", "/credits", "/disclaimer", "/contact", "/welcome", "/reads"];
    let under = |p: &str| path == p || path.starts_with(&format!("{p}/"));
    if path == "/" || path.is_empty() || EVERYONE.iter().any(|p| under(p)) {
        Access::Everyone
    } else if VISITORS.iter().any(|p| under(p)) {
        Access::Visitors
    } else {
        Access::Members
    }
}

/// Pages a visitor may open without an account.
pub fn is_open_to_visitors(path: &str) -> bool {
    access(path) != Access::Members
}

#[component]
pub fn AccountGate() -> impl IntoView {
    let location = use_location();
    let navigate = use_navigate();
    let signed_in = crate::services::account::signed_in();
    let features = LocalResource::new(|| async { crate::services::ai_engine::fetch_features().await });
    Effect::new(move |_| {
        let path = location.pathname.get();
        let member = signed_in.get();
        let replace = leptos_router::NavigateOptions { replace: true, ..Default::default() };
        match access(&path) {
            Access::Everyone => {}
            Access::Visitors => {
                if member {
                    navigate(&after_sign_in(), replace);
                }
            }
            Access::Members => {
                if member {
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
                navigate(&format!("/login?next={back}"), replace);
            }
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_address_has_one_audience() {
        for p in ["/login", "/signup", "/forgot"] {
            assert_eq!(access(p), Access::Visitors, "{p}");
        }
        for p in ["/", "/auth/confirm", "/title/tv-1399", "/help", "/help/article/share", "/privacy", "/terms", "/cookies", "/dmca", "/contact"] {
            assert_eq!(access(p), Access::Everyone, "{p}");
        }
        for p in ["/browse", "/browse/genre/83", "/settings/account", "/search", "/watch/tv/1399", "/loginx", "/helpdesk"] {
            assert_eq!(access(p), Access::Members, "{p}");
        }
    }
}
