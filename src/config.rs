//! Deployment settings. Every address and switch that differs between a laptop, a preview and production is here, and each
//! can be overridden when building, for example:
//!   STREAMS_RESOLVER_URL=https://resolver.example.com trunk build --release
//! so nothing environment specific is written into the pages. Names, contact addresses and logos are in brand.rs.

const fn setting(value: Option<&'static str>, default: &'static str) -> &'static str {
    match value {
        Some(v) => v,
        None => default,
    }
}

/// The public address of the site, used for canonical links and sharing.
pub const SITE_URL: &str = setting(option_env!("STREAMS_SITE_URL"), "https://streams.watch");

/// The address of the Help Centre. It has a host of its own (help.<domain>, one deployment with the app); `/help` on the main address is sent there.
pub const HELP_URL: &str = setting(option_env!("STREAMS_HELP_URL"), "https://help.streams.watch");

/// The host part of an address (`https://help.streams.watch/x` gives `help.streams.watch`).
pub fn host_of(url: &str) -> &str {
    url.split("://").nth(1).unwrap_or(url).split('/').next().unwrap_or("")
}

/// True when the page is open on the Help Centre's own host.
pub fn on_help_host() -> bool {
    web_sys::window().and_then(|w| w.location().hostname().ok()).is_some_and(|h| h == host_of(HELP_URL) || h.starts_with("help."))
}

/// True when the page is open on the main site's host (with or without www).
pub fn on_site_host() -> bool {
    web_sys::window().and_then(|w| w.location().hostname().ok()).is_some_and(|h| h == host_of(SITE_URL) || h == format!("www.{}", host_of(SITE_URL)))
}

/// A link into the app. On the Help Centre's host the app is another host, so the link is made absolute; elsewhere the plain path does.
pub fn app_link(path: &str) -> String {
    if on_help_host() { format!("{SITE_URL}{path}") } else { path.to_string() }
}

/// Stream resolver. It runs on a phone behind a Cloudflare tunnel (see streams-resolver-rs/deploy).
pub const RESOLVER_URL: &str = setting(option_env!("STREAMS_RESOLVER_URL"), "https://resolver.streams.watch");

/// Recommendation engine. The old variable name still works so existing build scripts keep working.
pub const ENGINE_URL: &str = setting(option_env!("STREAMS_ENGINE_URL"), setting(option_env!("STREAMS_ENGINE_URL"), "http://127.0.0.1:8088"));

/// The engine address for this page. A build that points at the laptop (127.0.0.1) is also used from another device on the same network (a phone opening the dev server by the
/// laptop's address): there the engine is on the same host as the page, at the same port. Everything else uses the address the build was made with.
pub fn engine_url() -> &'static str {
    use std::sync::OnceLock;
    static URL: OnceLock<String> = OnceLock::new();
    URL.get_or_init(|| {
        let built = ENGINE_URL.trim_end_matches('/');
        if built.contains("127.0.0.1") || built.contains("localhost") {
            if let Some(host) = web_sys::window().and_then(|w| w.location().hostname().ok()) {
                if host != "localhost" && host != "127.0.0.1" {
                    let port = built.rsplit(':').next().filter(|p| !p.is_empty() && p.chars().all(|c| c.is_ascii_digit())).unwrap_or("8088");
                    return format!("http://{host}:{port}");
                }
            }
        }
        built.to_string()
    })
}

/// Film and series catalog. The key is visible in the browser, as TMDB keys always are, and is free to replace.
pub const TMDB_API_URL: &str = setting(option_env!("STREAMS_TMDB_URL"), "https://api.themoviedb.org/3");
pub const TMDB_IMAGE_URL: &str = setting(option_env!("STREAMS_TMDB_IMAGE_URL"), "https://image.tmdb.org/t/p");
pub const TMDB_API_KEY: &str = setting(option_env!("STREAMS_TMDB_KEY"), "fc5fec3b73d8605daaeb1eb3b91157eb");

/// Accounts (Supabase). The key is the project's publishable key, which is meant to be public; the database rules protect the data.
pub const SUPABASE_URL: &str = setting(option_env!("STREAMS_SUPABASE_URL"), "https://emvyjhleyhimxxdqybsn.supabase.co");
pub const SUPABASE_KEY: &str = setting(option_env!("STREAMS_SUPABASE_KEY"), "sb_publishable_m286TDvNziieAgk--uDLlQ_MJQHgxNV");

/// Account and settings limits. They mirror the database rules in streams-backend-rs/supabase (five profiles not counting the
/// automatic Kids profile, 4 digit PINs, a password check no older than five minutes), so a change there is made here too.
pub const MAX_PROFILES: usize = 5;
pub const PROFILE_NAME_MAX: usize = 30;
pub const PIN_LENGTH: usize = 4;
pub const MIN_PASSWORD_LENGTH: usize = 8;
/// How long a password check counts for the screens that change account data.
pub const RECENT_SIGN_IN_SECONDS: u64 = 300;
pub const PHONE_DIGITS: std::ops::RangeInclusive<usize> = 6..=15;
pub const BLOCKED_TITLE_MAX: usize = 100;
pub const BLOCKED_TITLES_MAX: usize = 200;
/// Longest first or last name (account_details allows 60).
pub const NAME_MAX: usize = 60;
/// Titles listed on the Viewing activity screen before "Show more".
pub const VIEWING_PAGE: usize = 20;
/// Packages listed on the Open source licences screen before "Show more".
pub const LICENCES_PAGE: usize = 40;
/// Devices listed before "Show more".
pub const DEVICES_SHOWN: usize = 5;
/// Game handle length (profile_settings allows 3 to 16 letters and digits).
pub const GAME_HANDLE_MIN: usize = 3;
pub const GAME_HANDLE_MAX: usize = 16;
/// Length of the code sent by email for the password check. The Supabase project sends 8 digits (found 2026-10-18 when the 6 digit box refused the real code); if the project setting `mailer_otp_length` changes, change this too.
pub const CODE_LENGTH: usize = 8;
/// Minutes the emailed code stays valid (the Supabase default is one hour; the email template says the same number).
pub const CODE_VALID_MINUTES: u32 = 10;
/// Where the internet speed test downloads from, and how much. Override at build time with STREAMS_SPEED_TEST_URL.
pub const SPEED_TEST_URL: &str = setting(option_env!("STREAMS_SPEED_TEST_URL"), "https://speed.cloudflare.com/__down");
pub const SPEED_TEST_BYTES: u64 = 10_000_000;
/// How many security events the Access and devices screen lists.
pub const SECURITY_EVENTS_SHOWN: u32 = 10;

/// Addresses of pages other code links to, so a page is never written as a literal path inside a screen.
pub mod routes {
    pub const HOME: &str = "/";
    pub const BROWSE: &str = "/browse";
    pub const LOGIN: &str = "/login";
    pub const HELP: &str = "/help";
    pub const PRIVACY: &str = "/privacy";
    pub const TERMS: &str = "/terms";
    pub const DMCA: &str = "/dmca";
    pub const DISCLAIMER: &str = "/disclaimer";
    pub const HELP_CONTACT: &str = "/help/contact";
    pub const WATCH: &str = "/watch";
    /// Where the links in our emails land (the branded templates point here, not at supabase.co).
    pub const AUTH_CONFIRM: &str = "/auth/confirm";
}

/// Where the cookie pop-up sits from the `sm` width up (a foldable or a tablet). Set STREAMS_COOKIE_BANNER to top, bottom,
/// bottom-start, bottom-end or center when building, or add ?cookie_banner=top to the address to try one. A phone always gets a
/// full width sheet at the bottom.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BannerPosition {
    Top,
    Bottom,
    BottomStart,
    BottomEnd,
    Center,
}

impl BannerPosition {
    pub fn parse(value: &str) -> Option<Self> {
        Some(match value {
            "top" => Self::Top,
            "bottom" => Self::Bottom,
            "bottom-start" => Self::BottomStart,
            "bottom-end" => Self::BottomEnd,
            "center" => Self::Center,
            _ => return None,
        })
    }
}

pub fn cookie_banner_position() -> BannerPosition {
    let from_address = web_sys::window().and_then(|w| w.location().search().ok()).and_then(|q| q.trim_start_matches('?').split('&').find_map(|kv| kv.strip_prefix("cookie_banner=").map(str::to_string)));
    from_address.as_deref().or(option_env!("STREAMS_COOKIE_BANNER")).and_then(BannerPosition::parse).unwrap_or(BannerPosition::Center)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn banner_positions_parse() {
        assert_eq!(BannerPosition::parse("bottom-start"), Some(BannerPosition::BottomStart));
        assert_eq!(BannerPosition::parse("nowhere"), None);
    }
}
