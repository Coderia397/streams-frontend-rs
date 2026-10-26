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
    // `/signout` ends a session if there is one and goes to the start page either way, so a visitor is never sent to sign in first (and back to it)
    const EVERYONE: [&str; 7] = ["/title", "/auth", "/help", "/contact", "/welcome", "/reads", "/signout"];
    let under = |p: &str| path == p || path.starts_with(&format!("{p}/"));
    // the policies live under /help; their earlier addresses only move there (config::routes::MOVED)
    if path == "/" || path.is_empty() || EVERYONE.iter().any(|p| under(p)) || crate::config::routes::moved(path).is_some() {
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
    next.and_then(|n| safe_next(&n)).unwrap_or_else(|| crate::config::routes::BROWSE.to_string())
}

/// `next` when it is an address of this site that needs an account. Browsers read a backslash as a slash and drop tabs and line breaks,
/// so `/\example.com` or `/<tab>/example.com` would leave the site (a signed in member was sent to example.com, 2026-10-25): any of
/// those, or two leading slashes, is refused.
pub fn safe_next(n: &str) -> Option<String> {
    let local = n.starts_with('/') && !n.starts_with("//") && !n.contains('\\') && !n.chars().any(|c| c.is_control() || c.is_whitespace());
    (local && !is_open_to_visitors(n.split(['?', '#']).next().unwrap_or(""))).then(|| n.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_address_has_one_audience() {
        for p in ["/login", "/signup", "/forgot"] {
            assert_eq!(access(p), Access::Visitors, "{p}");
        }
        for p in ["/", "/auth/confirm", "/title/tv-1399", "/help", "/help/article/share", "/help/legal/privacy", "/help/legal/terms", "/privacy", "/privacy-policy", "/terms", "/cookies", "/dmca", "/contact"] {
            assert_eq!(access(p), Access::Everyone, "{p}");
        }
        for p in ["/browse", "/browse/genre/83", "/settings/account", "/search", "/watch/tv/1399", "/loginx", "/helpdesk"] {
            assert_eq!(access(p), Access::Members, "{p}");
        }
    }

    #[test]
    fn next_stays_on_the_site() {
        assert_eq!(safe_next("/browse?title=tv-1399").as_deref(), Some("/browse?title=tv-1399"));
        assert_eq!(safe_next("/settings/account").as_deref(), Some("/settings/account"));
        for bad in ["//example.com", "/\\example.com", "/\\/example.com", "/\t/example.com", "/\n/example.com", "https://example.com", "example.com", "/login", "/"] {
            assert_eq!(safe_next(bad), None, "{bad:?}");
        }
    }
}
