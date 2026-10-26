//! The Help Centre has a host of its own (help.<domain>), which cannot read the app's local storage, so it showed Browse and Sign in
//! to a signed in member (owner, 2026-10-26). The app keeps the name of the profile in use in one cookie on the site's domain, which both
//! hosts read, as help.netflix.com knows its member from netflix.com's cookies. The app writes it while an account is signed in or a
//! profile was chosen, and removes it otherwise (signing out); the Help Centre only reads it.

use wasm_bindgen::JsCast;

/// 400 days, the longest a browser keeps a cookie; the app writes it again whenever it starts.
const MAX_AGE: u32 = 400 * 24 * 3600;

fn document() -> Option<web_sys::HtmlDocument> {
    web_sys::window()?.document()?.dyn_into().ok()
}

/// `Domain=<site>` on the site and its hosts, so the Help Centre's host reads it too; a cookie of this host alone elsewhere (a preview, local work).
fn attributes(hostname: &str, https: bool) -> String {
    let site = crate::config::host_of(crate::config::SITE_URL);
    let domain = if hostname == site || hostname.ends_with(&format!(".{site}")) { format!("; Domain={site}") } else { String::new() };
    let secure = if https { "; Secure" } else { "" };
    format!("; Path=/{domain}; SameSite=Lax{secure}")
}

/// The name in `document.cookie` text, if the cookie is there.
fn parse(cookies: &str) -> Option<String> {
    let value = cookies.split(';').find_map(|c| c.trim().strip_prefix(&format!("{}=", crate::keys::HELLO)))?;
    let name = js_sys::decode_uri_component(value).ok()?.as_string()?;
    (!name.trim().is_empty()).then_some(name)
}

/// Keeps the cookie in step with the app: the profile's name, or no cookie.
pub fn write(name: Option<&str>) {
    let Some(doc) = document() else { return };
    let Some(location) = web_sys::window().map(|w| w.location()) else { return };
    let attrs = attributes(&location.hostname().unwrap_or_default(), location.protocol().ok().as_deref() == Some("https:"));
    let held = doc.cookie().ok().and_then(|c| parse(&c));
    if name.is_none() && held.is_none() {
        return;
    }
    let cookie = match name {
        Some(n) => format!("{}={}; Max-Age={MAX_AGE}{attrs}", crate::keys::HELLO, String::from(js_sys::encode_uri_component(n))),
        None => format!("{}=; Max-Age=0{attrs}", crate::keys::HELLO),
    };
    let _ = doc.set_cookie(&cookie);
}

/// The name of the profile in use in the app, when the app holds an account or a chosen profile.
pub fn read() -> Option<String> {
    parse(&document()?.cookie().ok()?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn both_hosts_of_the_site_share_the_cookie() {
        assert_eq!(attributes("streams.watch", true), "; Path=/; Domain=streams.watch; SameSite=Lax; Secure");
        assert_eq!(attributes("help.streams.watch", true), "; Path=/; Domain=streams.watch; SameSite=Lax; Secure");
        assert_eq!(attributes("localhost", false), "; Path=/; SameSite=Lax");
        assert_eq!(attributes("evilstreams.watch", true), "; Path=/; SameSite=Lax; Secure");
    }
}
