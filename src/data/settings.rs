//! What the settings screens list: age ratings, subtitle choices, data use choices, legal links, codec probes, licences.
//! Each entry names its translation key, so adding or removing a choice never touches a page and every label is translated.

use crate::config::routes;

/// A British (BBFC) age rating a profile can be limited to. `key` is what is saved (profile_settings.age_rating), `rank` orders
/// the ratings from the lowest, `kids` marks the ones a Kids profile may choose, and `db_maturity` is the value kept in the
/// older profiles.maturity column for the same step.
pub struct Rating {
    pub key: &'static str,
    pub label_key: &'static str,
    pub hint_key: &'static str,
    pub rank: u8,
    pub kids: bool,
    pub db_maturity: &'static str,
}

pub const RATINGS: &[Rating] = &[
    Rating { key: "U", label_key: "set-rating-u", hint_key: "set-rating-u-hint", rank: 0, kids: true, db_maturity: "little_kids" },
    Rating { key: "PG", label_key: "set-rating-pg", hint_key: "set-rating-pg-hint", rank: 1, kids: true, db_maturity: "older_kids" },
    Rating { key: "12", label_key: "set-rating-12", hint_key: "set-rating-12-hint", rank: 2, kids: false, db_maturity: "teens" },
    Rating { key: "15", label_key: "set-rating-15", hint_key: "set-rating-15-hint", rank: 3, kids: false, db_maturity: "teens" },
    Rating { key: "18", label_key: "set-rating-18", hint_key: "set-rating-18-hint", rank: 4, kids: false, db_maturity: "adult" },
];
/// No limit: everything up to 18 is shown.
pub const DEFAULT_RATING: &str = "18";
/// What a Kids profile starts with.
pub const DEFAULT_KIDS_RATING: &str = "PG";

pub fn rating(key: &str) -> &'static Rating {
    RATINGS.iter().find(|r| r.key == key).unwrap_or(&RATINGS[RATINGS.len() - 1])
}

/// The BBFC rank of a certificate as TMDB and the catalogue write it ("U", "PG", "12", "12A", "15", "18", "R18"), or None when
/// it is not one.
pub fn bbfc_rank(cert: &str) -> Option<u8> {
    match cert.trim().to_uppercase().as_str() {
        "U" | "UC" | "0" => Some(0),
        "PG" => Some(1),
        "12" | "12A" => Some(2),
        "15" => Some(3),
        "18" | "R18" => Some(4),
        _ => None,
    }
}

/// US certificates written as the nearest BBFC rank, for titles that only have an American one (TMDB "US" in the catalogue).
pub fn us_to_bbfc_rank(cert: &str) -> Option<u8> {
    match cert.trim().to_uppercase().as_str() {
        "G" | "TV-Y" | "TV-Y7" | "TV-G" => Some(0),
        "PG" | "TV-PG" => Some(1),
        "PG-13" => Some(2),
        "R" | "TV-14" => Some(3),
        "NC-17" | "TV-MA" => Some(4),
        _ => None,
    }
}

/// True when a title with these certificates may be shown to a profile limited to `max`. `gb` is the British certificate and
/// `us` the American one, either may be missing. A title with no usable certificate is shown only when nothing is limited.
pub fn rating_allows(max: &str, gb: Option<&str>, us: Option<&str>) -> bool {
    let limit = rating(max).rank;
    if limit >= rating(DEFAULT_RATING).rank {
        return true;
    }
    gb.and_then(bbfc_rank).or_else(|| us.and_then(us_to_bbfc_rank)).is_some_and(|r| r <= limit)
}

/// Subtitle colours: id (also the end of the translation key set-color-<id>) and the colour shown.
// colour: the colours a viewer may pick for subtitles (values the player applies), not the site's
pub const SUBTITLE_COLORS: &[(&str, &str)] = &[("white", "#ffffff"), ("yellow", "#ffe600"), ("green", "#2fd45a"), ("cyan", "#22d3ee"), ("blue", "#3b82f6"), ("magenta", "#e879f9"), ("red", "#ef4444"), ("black", "#000000")];
/// Text edge styles; the key is set-edge-<id>.
pub const SUBTITLE_EDGES: &[&str] = &["none", "raised", "depressed", "uniform", "shadow"];
/// Text sizes: value, translation key and the size in pixels used by the preview.
pub const SUBTITLE_SIZES: &[(u8, &str, u32)] = &[(0, "set-sub-size-small", 16), (1, "set-sub-size-medium", 22), (2, "set-sub-size-large", 30)];

/// Data use choices: value saved and translation key.
pub const DATA_USAGE: &[(&str, &str)] = &[("auto", "set-data-auto"), ("save", "set-data-save"), ("max", "set-data-max")];

/// Data use per profile (the website's "Data usage per screen"): value saved, label and hint keys.
pub const PROFILE_DATA_USAGE: &[(&str, &str, &str)] = &[
    ("auto", "set-usage-auto", "set-usage-auto-hint"),
    ("low", "set-usage-low", "set-usage-low-hint"),
    ("medium", "set-usage-medium", "set-usage-medium-hint"),
    ("high", "set-usage-high", "set-usage-high-hint"),
];

/// What a profile can be told about, and by which channel. The saved key is "<channel>.<topic>"; the channel alone ("email",
/// "push") is its master switch. Label and hint keys are set-topic-<topic> and set-topic-<topic>-hint.
pub const NOTIFICATION_CHANNELS: &[(&str, &str, &str)] = &[("email", "set-notify-email", "set-notify-email-hint"), ("push", "set-notify-push", "set-notify-push-hint")];
pub const NOTIFICATION_TOPICS: &[(&str, &[&str])] = &[("new_arrivals", &["email", "push"]), ("recommendations", &["email", "push"]), ("reminders", &["email", "push"]), ("news", &["email"])];

/// Words a suggested game handle is put together from, with a number at the end.
pub const HANDLE_ADJECTIVES: &[&str] = &["Bold", "Swift", "Quiet", "Lucky", "Brave", "Clever", "Mellow", "Bright", "Wild", "Steady", "Cosmic", "Gentle"];
pub const HANDLE_NOUNS: &[&str] = &["Fox", "Crane", "Otter", "Falcon", "Panda", "Heron", "Lynx", "Raven", "Tiger", "Whale", "Comet", "Maple"];

/// Browsers and systems recognised in a session's user agent, first match wins: text to find and the name shown.
pub const BROWSERS: &[(&str, &str)] = &[("Edg/", "Edge"), ("OPR/", "Opera"), ("Firefox/", "Firefox"), ("Chrome/", "Chrome"), ("CriOS/", "Chrome"), ("Safari/", "Safari")];
pub const SYSTEMS: &[(&str, &str)] = &[("iPhone", "iPhone"), ("iPad", "iPad"), ("Android", "Android"), ("Windows", "Windows"), ("Mac OS X", "Mac"), ("CrOS", "ChromeOS"), ("Linux", "Linux")];

/// Legal pages listed under App settings: icon, translation key and address. An empty address opens the cookie preferences.
pub const LEGAL_LINKS: &[(&str, &str, &str)] = &[
    ("file-text", "set-app-licences", "/settings/app/licences"),
    ("file-text", "help-ui-legal-privacy", routes::PRIVACY),
    ("cookie", "cookie-link", ""),
    ("file-text", "help-ui-legal-terms", routes::TERMS),
    ("file-text", "help-ui-legal-dmca", routes::DMCA),
    ("file-text", "help-ui-legal-disclaimer", routes::DISCLAIMER),
];

/// Codecs the playback specification checks: name and the media type asked of the browser.
pub const CODEC_PROBES: &[(&str, &str)] = &[
    ("AVC (H.264)", "video/mp4; codecs=\"avc1.640028\""),
    ("HEVC (H.265)", "video/mp4; codecs=\"hvc1.1.6.L120.90\""),
    ("AV1", "video/mp4; codecs=\"av01.0.08M.08\""),
    ("VP9", "video/webm; codecs=\"vp09.00.10.08\""),
];

/// Widevine probe for the playback specification: key system, the media type and init data type asked for, and the
/// robustness levels tried from best to worst with the level name shown.
pub const DRM_KEY_SYSTEM: &str = "com.widevine.alpha";
pub const DRM_PROBE_MIME: &str = "video/mp4; codecs=\"avc1.640028\"";
pub const DRM_INIT_DATA: &str = "cenc";
pub const DRM_LEVELS: &[(&str, &str)] = &[("HW_SECURE_ALL", "L1"), ("SW_SECURE_DECODE", "L3")];
pub const DRM_NAME: &str = "Widevine";

/// Largest screen side in pixels and the name of the playback quality it supports, best first.
pub const RESOLUTION_LABELS: &[(u32, &str)] = &[(3840, "4K"), (2560, "QHD"), (1920, "Full HD"), (1280, "HD")];

/// What the "Download my data" file is called and its media type.
pub const EXPORT_MIME: &str = "application/json";
pub fn export_file_name() -> String {
    export_file_name_for("data", "json")
}

/// "<brand>-<what>.<extension>", for example streams-viewing-activity.csv.
pub fn export_file_name_for(what: &str, extension: &str) -> String {
    format!("{}-{what}.{extension}", crate::brand::NAME.to_lowercase())
}

/// Subtitle preview: corner radius, padding and window glow in pixels (the preview box only, not the player).
pub const PREVIEW_RADIUS_PX: u32 = 4;
pub const PREVIEW_PAD_Y_PX: u32 = 2;
pub const PREVIEW_PAD_X_PX: u32 = 8;
pub const PREVIEW_WINDOW_PX: u32 = 8;

/// Every package the app is built from (Rust crates, JavaScript packages and build tools) with its licence, generated by
/// scripts/make_licences.py from Cargo and npm.
pub const LICENCES_JSON: &str = include_str!("licences.json");

#[derive(serde::Deserialize, Clone, PartialEq, Debug)]
pub struct Package {
    pub name: String,
    pub version: String,
    pub license: String,
    pub kind: String,
}

pub fn licences() -> Vec<Package> {
    serde_json::from_str(LICENCES_JSON).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_licence_list_is_generated_and_complete() {
        let list = licences();
        assert!(list.len() > 100, "run scripts/make_licences.py");
        assert!(list.iter().any(|p| p.name == "leptos" && p.kind == "rust"));
        assert!(list.iter().any(|p| p.name == "hls.js" && p.kind == "js"));
        assert!(list.iter().all(|p| !p.license.is_empty()));
    }

    #[test]
    fn ratings_are_the_british_scale_in_order() {
        let keys: Vec<_> = RATINGS.iter().map(|r| r.key).collect();
        assert_eq!(keys, ["U", "PG", "12", "15", "18"]);
        assert!(RATINGS.windows(2).all(|w| w[0].rank < w[1].rank));
        assert!(RATINGS.iter().filter(|r| r.kids).count() == 2);
        assert_eq!(rating("nonsense").key, DEFAULT_RATING);
    }

    #[test]
    fn a_limit_hides_what_is_above_it() {
        assert!(rating_allows("18", Some("18"), None));
        assert!(rating_allows("18", None, None), "no limit shows everything, rated or not");
        assert!(rating_allows("PG", Some("U"), None));
        assert!(rating_allows("PG", Some("PG"), None));
        assert!(!rating_allows("PG", Some("12"), None));
        assert!(rating_allows("15", Some("12A"), None));
        assert!(!rating_allows("15", Some("R18"), None));
        // no British certificate: the American one is mapped
        assert!(rating_allows("12", None, Some("PG-13")));
        assert!(!rating_allows("12", None, Some("R")));
        // nothing known about the title: hidden when a limit is set
        assert!(!rating_allows("PG", None, None));
    }
}
