//! The account gate: one rule for every address. Each address belongs to visitors (signing in, signing up, the forgotten password page),
//! to everyone (the landing page, a title's own page, help, the policies, the links in account emails), or to members (the rest of the
//! app). The gate follows the address and the signed in signal together (`account::signed_in`, which also changes when another tab signs
//! in or out): a member on a visitors' page goes where they were heading, or Home, as Netflix sends a signed in member away from its sign
//! in page; a visitor on a members' page goes to sign in, which sends them back after they sign in. The engine can switch the members'
//! side off for a while (`accounts_required: false` in `features.json`, read by `/api/features`); an engine that cannot be reached counts
//! as "required". The rules are here; `routing::decide` applies them, after the host, moved and unknown addresses (note 139).


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
    const EVERYONE: [&str; 8] = ["/title", "/auth", "/help", "/contact", "/welcome", "/reads", "/signout", "/NotFound"];
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

/// Where to go after signing in: the address the gate took the person from (only an address of this site), otherwise Home.
pub fn after_sign_in() -> String {
    after_sign_in_from(&web_sys::window().and_then(|w| w.location().search().ok()).unwrap_or_default())
}

/// `after_sign_in` for a given query (`?next=...`).
pub fn after_sign_in_from(search: &str) -> String {
    let next = search.trim_start_matches('?').split('&').find_map(|kv| kv.strip_prefix("next=")).map(decode);
    next.and_then(|n| safe_next(&n)).unwrap_or_else(|| crate::config::routes::BROWSE.to_string())
}

fn decode(v: &str) -> String {
    #[cfg(target_arch = "wasm32")]
    {
        js_sys::decode_uri_component(v).ok().and_then(|s| s.as_string()).unwrap_or_default()
    }
    // the same as decodeURIComponent, for the tests on the host
    #[cfg(not(target_arch = "wasm32"))]
    {
        let bytes = v.as_bytes();
        let mut out = Vec::with_capacity(bytes.len());
        let mut i = 0;
        while i < bytes.len() {
            if bytes[i] == b'%' && i + 2 < bytes.len() {
                if let Ok(b) = u8::from_str_radix(&v[i + 1..i + 3], 16) {
                    out.push(b);
                    i += 3;
                    continue;
                }
            }
            out.push(bytes[i]);
            i += 1;
        }
        String::from_utf8(out).unwrap_or_default()
    }
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
        for p in ["/", "/NotFound", "/auth/confirm", "/title/tv-1399", "/help", "/help/article/share", "/help/legal/privacy", "/help/legal/terms", "/privacy", "/privacy-policy", "/terms", "/cookies", "/dmca", "/contact"] {
            assert_eq!(access(p), Access::Everyone, "{p}");
        }
        for p in ["/browse", "/browse/genre/83", "/account", "/settings/p1", "/password", "/search", "/watch/tv/1399", "/loginx", "/helpdesk"] {
            assert_eq!(access(p), Access::Members, "{p}");
        }
    }

    #[test]
    fn next_stays_on_the_site() {
        assert_eq!(safe_next("/browse?title=tv-1399").as_deref(), Some("/browse?title=tv-1399"));
        assert_eq!(safe_next("/account").as_deref(), Some("/account"));
        for bad in ["//example.com", "/\\example.com", "/\\/example.com", "/\t/example.com", "/\n/example.com", "https://example.com", "example.com", "/login", "/"] {
            assert_eq!(safe_next(bad), None, "{bad:?}");
        }
    }
}
