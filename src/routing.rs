//! Where an address goes: one ordered list of rules (`decide`), applied in one place (`AddressKeeper`, mounted once in lib.rs). Pages draw;
//! they never move the address because of what it is. Five parts used to move it on their own (the host guard, the account gate, the
//! routes' fallback, the settings page, the redirects inside routes) and reacted to each other's moves while their pages were being taken
//! down: a visitor's /account went to sign in and then to "Lost your way?" (2026-10-27), and twice before the app aborted (note 139).
use crate::components::layout::account_gate::{access, after_sign_in_from, Access};
use crate::config::{routes, HELP_URL, SITE_URL};
use crate::pages::settings::Screen;
use leptos::prelude::*;

/// What to do with the address in the bar.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Step {
    Stay,
    /// Not known yet (the engine has not said whether a visitor needs an account): nothing is drawn and nothing moves.
    Wait,
    /// Another address of this app, in place of this one (the back button skips it).
    Replace(String),
    /// A full address on the other host, loaded afresh.
    Leave(String),
}

/// Which of the two hosts the page is on; `Other` is a preview or this machine, where both kinds of page are served.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Host {
    Site,
    Help,
    Other,
}

/// What the rules depend on besides the address.
#[derive(Clone, Debug)]
pub struct Who {
    pub host: Host,
    /// The page's origin, for the address "Lost your way?" names (`prev`).
    pub origin: String,
    pub member: bool,
    /// Whether the engine requires an account for the app; None while it has not answered (the gate waits).
    pub accounts_required: Option<bool>,
    pub phone: bool,
    /// A browser that signed in or chose a profile before: the root opens Home.
    pub returning: bool,
    /// This machine or its network: the developer pages exist (note 67 E8).
    pub local: bool,
    /// The profile in use, for the settings addresses of before that named none.
    pub active: String,
    /// Whether the profile in use is a Kids profile, and how many Kids profiles the browser holds: which of the two homes is its own.
    pub kids_active: bool,
    pub kids_profiles: usize,
}

/// The rules, in order: the host, an address that moved, an address the site does not have, who may open it. `search` is the query with
/// its `?` (or empty).
pub fn decide(path: &str, search: &str, who: &Who) -> Step {
    let help_path = path == routes::HELP || path.starts_with("/help/");
    // 1. the Help Centre's pages on its own host, everything else on the main one (functions/_middleware.js does the same at the edge)
    match who.host {
        Host::Site if help_path => return Step::Leave(format!("{HELP_URL}{path}{search}")),
        Host::Help if !help_path && path != routes::HOME => return Step::Leave(format!("{SITE_URL}{path}{search}")),
        _ => {}
    }
    // 2. an address that moved
    if let Some(to) = moved(path, search, who) {
        return Step::Replace(to);
    }
    // 3. an address the site does not have: Netflix's "Lost your way?" with the address asked (pages/not_found.rs)
    if !known(path, who) {
        let asked = format!("{}{path}{search}", who.origin);
        return Step::Replace(format!("{}?prev={}", routes::NOT_FOUND, encode(&asked)));
    }
    // 4. who may open it (components/layout/account_gate.rs)
    match access(path) {
        Access::Everyone => Step::Stay,
        Access::Visitors if who.member => Step::Replace(after_sign_in_from(search)),
        Access::Visitors => Step::Stay,
        Access::Members if who.member || who.accounts_required == Some(false) => Step::Stay,
        Access::Members if who.accounts_required.is_none() => Step::Wait,
        Access::Members => Step::Replace(format!("{}?next={}", routes::LOGIN, encode(&format!("{path}{search}")))),
    }
}

/// Where an address that moved goes now, with its query, or None.
fn moved(path: &str, search: &str, who: &Who) -> Option<String> {
    let path = path.trim_end_matches('/');
    let path = if path.is_empty() { routes::HOME } else { path };
    // the policies' earlier addresses and the ones people type (config::routes::MOVED)
    if let Some(to) = routes::moved(path) {
        return Some(format!("{to}{search}"));
    }
    // the named addresses of before: Series is genre 83 and Films is genre 34399
    let named = match path {
        "/browse/movies" | "/browse/films" | "/movies" => Some("/browse/genre/34399"),
        "/browse/shows" | "/browse/series" | "/tv" => Some("/browse/genre/83"),
        "/new" => Some("/latest"),
        "/list" => Some(routes::MY_LIST),
        "/browse/language" => Some("/browse/original-audio"),
        "/contact" => Some(routes::HELP_CONTACT),
        "/welcome" => Some(routes::HOME),
        _ => None,
    };
    if let Some(to) = named {
        return Some(format!("{to}{search}"));
    }
    // the root: the Help Centre on its host, Home for a returning visitor, the landing page otherwise
    if path == routes::HOME {
        return match who.host {
            Host::Help => Some(routes::HELP.to_string()),
            _ if who.returning => Some(format!("{}{search}", routes::BROWSE)),
            _ => None,
        };
    }
    // the two homes: a Kids profile lives at /kids, the others at /browse; /kids for another profile switches to the Kids profile there
    // (lib.rs HomeRoute), unless there is none to switch to. The query goes along: a shared link's `?title=` opens on either home
    if path == routes::BROWSE && who.kids_active {
        return Some(format!("{}{search}", routes::KIDS));
    }
    if path == routes::KIDS && !who.kids_active && (who.kids_profiles == 0 || who.active.is_empty()) {
        return Some(format!("{}{search}", routes::BROWSE));
    }
    // Clips and Notifications are the phone's alone: a computer or tablet goes Home, also when a phone becomes one (note 116)
    if !who.phone && (path == "/clips" || path == "/notifications") {
        return Some(routes::BROWSE.to_string());
    }
    // a title's own page is for visitors: a member sees it in the details window on Home
    if let Some(slug) = path.strip_prefix("/title/") {
        if who.member && crate::pages::title::parse_slug(slug).is_some() {
            return Some(format!("{}?title={slug}", routes::BROWSE));
        }
    }
    // the settings addresses of before 2026-10-27 (note 138)
    if Screen::is_old(path) {
        return Screen::parse(path, &who.active, |_| false).map(|s| format!("{}{search}", s.path()));
    }
    None
}

/// Whether the app has a page at `path`. It lists the routes of lib.rs (a test keeps the two equal); the addresses that move are not here.
pub fn known(path: &str, who: &Who) -> bool {
    let parts: Vec<&str> = path.split('/').filter(|p| !p.is_empty()).collect();
    match parts.as_slice() {
        [] => true,
        ["browse"] | ["kids"] | ["browse", "my-list"] | ["browse", "games"] | ["games"] | ["browse", "original-audio"] | ["latest"] | ["search"] | ["reads"] => true,
        ["notifications"] | ["clips"] | ["profiles", "manage"] | ["login"] | ["signout"] | ["signup"] | ["forgot"] | ["auth", "confirm"] | ["NotFound"] => true,
        ["browse", "genre", _] | ["watch", _] | ["watch", _, _] => true,
        ["title", slug] => crate::pages::title::parse_slug(slug).is_some(),
        ["help"] | ["help", "search"] | ["help", "contact"] | ["help", "article", _] => true,
        ["help", "legal", doc] => ["privacy", "terms", "cookies", "dmca", "credits", "disclaimer"].contains(doc),
        // the badge sheet is in development builds only (pages/dev_badges.rs)
        ["dev", "badges"] => who.local && cfg!(debug_assertions),
        ["dev", "logos"] => who.local,
        _ => Screen::owns(path) && Screen::parse(path, "", |_| false).is_some(),
    }
}

fn encode(s: &str) -> String {
    #[cfg(target_arch = "wasm32")]
    {
        String::from(js_sys::encode_uri_component(s))
    }
    // the same as encodeURIComponent, for the tests on the host
    #[cfg(not(target_arch = "wasm32"))]
    {
        s.bytes().map(|b| if b.is_ascii_alphanumeric() || b"-_.!~*'()".contains(&b) { (b as char).to_string() } else { format!("%{b:02X}") }).collect()
    }
}

/// True when the page is opened from this machine or a private network address (localhost, 127.0.0.1, 10.x, 192.168.x, 172.16 to 31.x, *.local): where the developer pages may show.
pub fn is_local_host() -> bool {
    let host = web_sys::window().and_then(|w| w.location().hostname().ok()).unwrap_or_default();
    let private_172 = host.strip_prefix("172.").and_then(|r| r.split('.').next()).and_then(|n| n.parse::<u8>().ok()).is_some_and(|n| (16..=31).contains(&n));
    host == "localhost" || host == "127.0.0.1" || host == "[::1]" || host.starts_with("10.") || host.starts_with("192.168.") || private_172 || host.ends_with(".local")
}

/// Applies `decide` whenever the address or anything it depends on changes, and makes the one move. Mounted once, inside the router and
/// around the routes (`children`), which it draws only while the address stays: a page is never drawn for an address that is about to
/// move, so it is never taken down mid-start with its timers and loads still running (a visitor's /browse on its way to sign in panicked
/// four times on disposed signals, 2026-10-27). It moves after drawing, never during it (that aborted the app, 2026-10-25).
#[component]
pub fn AddressKeeper(children: ChildrenFn) -> impl IntoView {
    let location = leptos_router::hooks::use_location();
    let navigate = leptos_router::hooks::use_navigate();
    let signed_in = crate::services::account::signed_in();
    let features = LocalResource::new(|| async { crate::services::ai_engine::fetch_features().await });
    let form = crate::form::use_form();
    let profiles = crate::store::use_profile_store();
    let host = if crate::config::on_help_host() { Host::Help } else if crate::config::on_site_host() { Host::Site } else { Host::Other };
    let origin = web_sys::window().and_then(|w| w.location().origin().ok()).unwrap_or_default();
    let step = Memo::new(move |_| {
        let path = location.pathname.get();
        let search = location.search.get();
        let search = if search.is_empty() || search.starts_with('?') { search } else { format!("?{search}") };
        let who = Who {
            host,
            origin: origin.clone(),
            member: signed_in.get(),
            // an engine that cannot be reached answers "required"; the switch is off only when it has said so
            accounts_required: features.get().map(|flags| flags.get("accounts_required") != Some(&false)),
            phone: form.get().is_phone(),
            returning: crate::services::account::is_returning(),
            local: is_local_host(),
            active: profiles.active_profile_id.get().unwrap_or_default(),
            kids_active: profiles.profiles.with(|l| profiles.active_profile_id.with(|id| l.iter().any(|p| p.is_kids && Some(&p.id) == id.as_ref()))),
            kids_profiles: profiles.profiles.with(|l| l.iter().filter(|p| p.is_kids).count()),
        };
        decide(&path, &search, &who)
    });
    Effect::new(move |_| {
        match step.get() {
            Step::Stay | Step::Wait => {}
            Step::Replace(to) => navigate(&to, leptos_router::NavigateOptions { replace: true, ..Default::default() }),
            Step::Leave(url) => {
                if let Some(w) = web_sys::window() {
                    let _ = w.location().replace(&url);
                }
            }
        }
    });
    view! { <Show when=move || step.with(|s| *s == Step::Stay)>{children()}</Show> }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn who() -> Who {
        Who { host: Host::Site, origin: "https://streams.watch".into(), member: false, accounts_required: Some(true), phone: false, returning: false, local: false, active: "p1".into(), kids_active: false, kids_profiles: 1 }
    }

    fn member() -> Who {
        Who { member: true, returning: true, ..who() }
    }

    /// Follows the moves inside the app until the address stays; None if it never does within 5 steps.
    fn settle(address: &str, w: &Who) -> Option<String> {
        let mut at = address.to_string();
        for _ in 0..5 {
            let (path, search) = at.split_once('?').map(|(p, s)| (p.to_string(), format!("?{s}"))).unwrap_or((at.clone(), String::new()));
            match decide(&path, &search, w) {
                Step::Stay | Step::Wait => return Some(at),
                Step::Replace(to) => at = to,
                Step::Leave(url) => return Some(url),
            }
        }
        None
    }

    #[test]
    fn a_visitor_is_sent_to_sign_in_once_and_stays_there() {
        assert_eq!(settle("/account", &who()).as_deref(), Some("/login?next=%2Faccount"));
        assert_eq!(settle("/settings/p1", &who()).as_deref(), Some("/login?next=%2Fsettings%2Fp1"));
        assert_eq!(settle("/password", &who()).as_deref(), Some("/login?next=%2Fpassword"));
        // the engine has not answered yet: nothing is drawn and nothing moves
        assert_eq!(decide("/account", "", &Who { accounts_required: None, ..who() }), Step::Wait);
        assert_eq!(decide("/account", "", &Who { accounts_required: None, ..member() }), Step::Stay);
        assert_eq!(decide("/", "", &Who { accounts_required: None, ..who() }), Step::Stay);
        // the engine switched accounts off
        assert_eq!(settle("/browse", &Who { accounts_required: Some(false), ..who() }).as_deref(), Some("/browse"));
    }

    #[test]
    fn a_member_goes_on_from_sign_in() {
        assert_eq!(settle("/login?next=%2Faccount", &member()).as_deref(), Some("/account"));
        assert_eq!(settle("/login", &member()).as_deref(), Some("/browse"));
        assert_eq!(settle("/title/tv-1399", &member()).as_deref(), Some("/browse?title=tv-1399"));
        assert_eq!(settle("/title/tv-1399", &who()).as_deref(), Some("/title/tv-1399"));
    }

    #[test]
    fn an_address_the_site_does_not_have_is_lost_once() {
        for w in [who(), member()] {
            assert_eq!(settle("/account/nosuchpage", &w).as_deref(), Some("/NotFound?prev=https%3A%2F%2Fstreams.watch%2Faccount%2Fnosuchpage"));
            assert_eq!(settle("/nosuchpage", &w).as_deref(), Some("/NotFound?prev=https%3A%2F%2Fstreams.watch%2Fnosuchpage"));
            assert_eq!(settle("/title/notatitle", &w).as_deref(), Some("/NotFound?prev=https%3A%2F%2Fstreams.watch%2Ftitle%2Fnotatitle"));
            assert_eq!(settle("/dev/badges", &w).as_deref(), Some("/NotFound?prev=https%3A%2F%2Fstreams.watch%2Fdev%2Fbadges"));
        }
        assert_eq!(settle("/dev/badges", &Who { local: true, host: Host::Other, ..member() }).as_deref(), Some("/dev/badges"));
    }

    #[test]
    fn moved_addresses_move() {
        let m = member();
        assert_eq!(settle("/tv", &m).as_deref(), Some("/browse/genre/83"));
        assert_eq!(settle("/movies", &m).as_deref(), Some("/browse/genre/34399"));
        assert_eq!(settle("/list", &m).as_deref(), Some("/browse/my-list"));
        assert_eq!(settle("/welcome", &m).as_deref(), Some("/browse"));
        assert_eq!(settle("/", &who()).as_deref(), Some("/"));
        assert_eq!(settle("/settings/account/security", &m).as_deref(), Some("/account/security"));
        assert_eq!(settle("/settings/profile/p2/edit", &m).as_deref(), Some("/settings/profile/edit/p2"));
        assert_eq!(settle("/clips", &m).as_deref(), Some("/browse"));
        assert_eq!(settle("/clips", &Who { phone: true, ..m.clone() }).as_deref(), Some("/clips"));
        // a policy's earlier address goes to the policy, which lives on the Help Centre's host
        assert_eq!(settle("/privacy", &m).as_deref(), Some("https://help.streams.watch/help/legal/privacy"));
    }

    #[test]
    fn each_kind_of_profile_has_its_own_home() {
        let kid = Who { kids_active: true, ..member() };
        assert_eq!(settle("/browse?title=tv-1399", &kid).as_deref(), Some("/kids?title=tv-1399"));
        assert_eq!(settle("/kids", &kid).as_deref(), Some("/kids"));
        // another profile on /kids switches to the Kids profile there (HomeRoute); with none to switch to it goes to /browse
        assert_eq!(settle("/kids", &member()).as_deref(), Some("/kids"));
        assert_eq!(settle("/kids", &Who { kids_profiles: 0, ..member() }).as_deref(), Some("/browse"));
    }

    #[test]
    fn each_host_keeps_its_own_pages() {
        assert_eq!(decide("/help/contact", "?a=1", &who()), Step::Leave("https://help.streams.watch/help/contact?a=1".into()));
        let help = Who { host: Host::Help, ..who() };
        assert_eq!(decide("/browse", "", &help), Step::Leave("https://streams.watch/browse".into()));
        assert_eq!(settle("/", &help).as_deref(), Some("/help"));
        assert_eq!(decide("/help", "", &help), Step::Stay);
    }

    #[test]
    fn every_route_in_lib_is_known_or_moves() {
        let lib = include_str!("lib.rs");
        let w = Who { local: true, phone: true, host: Host::Other, ..member() };
        let mut seen = 0;
        for piece in lib.split("path!(\"").skip(1) {
            let route = piece.split('"').next().unwrap_or_default();
            let path = route
                .replace("/account/*any", "/account/security")
                .replace("/notificationSettings/*any", "/notificationSettings/email/p1")
                .replace("*any", "p1")
                .replace("/title/:slug", "/title/tv-1399")
                .replace(":slug", "share")
                .replace(":kind", "tv")
                .replace(":id", "1");
            let path = if path.is_empty() { "/".to_string() } else { path };
            assert!(known(&path, &w) || moved(&path, "", &w).is_some(), "the route {route} is neither known to routing::known nor moved");
            seen += 1;
        }
        assert!(seen > 40, "only {seen} routes read from lib.rs");
    }
}
