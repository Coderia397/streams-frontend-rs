//! Help center structure: topics hold groups, groups hold articles. The words live in i18n/en.ftl under
//! help-topic-<id>, help-group-<id> and help-art-<slug>-(title|summary|keywords|body); this file holds only ids and links,
//! so adding an article is one line here plus its four messages in the .ftl file.

pub struct Topic {
    pub id: &'static str,
    pub icon: &'static str,
    /// Text color class for the icon.
    pub tone: &'static str,
    pub groups: &'static [Group],
}

pub struct Group {
    pub id: &'static str,
    pub articles: &'static [&'static str],
}

/// A policy shown inside the help center. Its text is in i18n/en.ftl as legal-<id>-title, legal-<id>-updated,
/// legal-<id>-<n>-title, legal-<id>-<n>-body and optional legal-<id>-<n>-b<k> bullets.
pub struct LegalDoc {
    pub id: &'static str,
    pub path: &'static str,
    /// Number of numbered sections.
    pub sections: u8,
}

pub const LEGAL: &[LegalDoc] = &[
    LegalDoc { id: "privacy", path: "/privacy", sections: 14 },
    LegalDoc { id: "terms", path: "/terms", sections: 18 },
    LegalDoc { id: "cookie-policy", path: "/cookies", sections: 4 },
    LegalDoc { id: "dmca", path: "/dmca", sections: 6 },
    LegalDoc { id: "disclaimer", path: "/disclaimer", sections: 5 },
    LegalDoc { id: "credits", path: "/credits", sections: 6 },
];

pub fn legal_doc(id: &str) -> Option<&'static LegalDoc> {
    LEGAL.iter().find(|d| d.id == id)
}

/// Group entries that start with this prefix are policies, not articles: "doc:privacy".
pub const DOC_PREFIX: &str = "doc:";

pub struct Article {
    pub slug: &'static str,
    /// Articles shown in the Related Articles card, in order.
    pub related: &'static [&'static str],
}

/// Shown under the search box on the help home, in this order.
pub const POPULAR: &[&str] = &["what-is", "wont-play", "profiles-manage", "kids-profile", "need-account"];

/// Quick links on the contact page: (message id suffix, icon, link). Article links start with /help/article/.
pub const QUICK_LINKS: &[(&str, &str, &str)] = &[
    ("profiles", "ph-user-circle", "/settings/profiles"),
    ("language", "ph-translate", "/settings"),
    ("signin", "ph-sign-in", "/login"),
    ("copyright", "ph-copyright", "/help/article/copyright"),
    ("title", "ph-megaphone", "/help/article/title-feedback"),
    ("email", "ph-envelope-simple", "mailto"),
];

pub const TOPICS: &[Topic] = &[
    Topic {
        id: "account",
        icon: "ph-user",
        tone: "text-accent",
        groups: &[
            Group { id: "profiles", articles: &["profiles-manage", "switch-profile", "profile-pin", "kids-profile", "parental-controls", "profile-icons", "transfer-profile"] },
            Group { id: "signing-in", articles: &["need-account", "sign-in", "forgot-password", "change-email-password", "devices-sign-out"] },
            Group { id: "account-data", articles: &["saved-data", "clear-data", "download-data", "delete-account", "privacy-data", "cookies-explained"] },
            Group { id: "paying", articles: &["free-to-use"] },
        ],
    },
    Topic {
        id: "fix",
        icon: "ph-wrench",
        tone: "text-accent",
        groups: &[
            Group { id: "playback-issues", articles: &["wont-play", "playback-unavailable", "black-screen", "buffering", "subtitles-missing", "no-sound", "autoplay-blocked"] },
            Group { id: "device-issues", articles: &["supported-browsers", "fix-phone", "fix-tv", "extensions-blockers", "vpn-network"] },
            Group { id: "page-issues", articles: &["rows-wont-load", "list-missing", "forgot-pin", "stuck-loading", "slow-pages", "is-down"] },
        ],
    },
    Topic {
        id: "watching",
        icon: "ph-devices",
        tone: "text-brand",
        groups: &[
            Group { id: "features", articles: &["language", "subtitles", "shortcuts", "skip-intro-next", "accessibility", "episodes"] },
            Group { id: "library", articles: &["my-list", "continue-watching", "viewing-history", "notifications", "share"] },
            Group { id: "finding", articles: &["search-browse", "browse-language", "new-popular", "recommendations", "trailers-clips", "genres", "top-10", "title-details", "clips"] },
        ],
    },
    Topic {
        id: "start",
        icon: "ph-rocket-launch",
        tone: "text-brand-deep",
        groups: &[
            Group { id: "about", articles: &["what-is", "getting-started", "sources", "contribute", "translate"] },
            Group { id: "setup", articles: &["install-app", "phone-tablet", "watch-on-tv"] },
            Group { id: "reporting", articles: &["copyright", "title-feedback", "report-bug", "fake-sites", "security-report"] },
        ],
    },
    Topic {
        id: "legal",
        icon: "ph-scales",
        tone: "text-brand",
        groups: &[Group { id: "policies", articles: &["doc:privacy", "doc:terms", "doc:cookie-policy", "doc:dmca", "doc:disclaimer", "doc:credits"] }],
    },
];

pub const ARTICLES: &[Article] = &[
    Article { slug: "profiles-manage", related: &["switch-profile", "profile-pin", "kids-profile", "parental-controls", "transfer-profile"] },
    Article { slug: "switch-profile", related: &["profiles-manage", "profile-pin", "kids-profile", "forgot-pin", "saved-data"] },
    Article { slug: "profile-pin", related: &["forgot-pin", "kids-profile", "profiles-manage", "switch-profile", "saved-data"] },
    Article { slug: "kids-profile", related: &["parental-controls", "profile-pin", "profiles-manage", "switch-profile", "recommendations"] },
    Article { slug: "need-account", related: &["sign-in", "free-to-use", "profiles-manage", "saved-data", "privacy-data"] },
    Article { slug: "sign-in", related: &["need-account", "forgot-password", "devices-sign-out", "change-email-password", "saved-data"] },
    Article { slug: "saved-data", related: &["clear-data", "privacy-data", "need-account", "list-missing", "viewing-history"] },
    Article { slug: "clear-data", related: &["saved-data", "delete-account", "viewing-history", "privacy-data", "sign-in"] },
    Article { slug: "privacy-data", related: &["download-data", "delete-account", "saved-data", "recommendations", "cookies-explained"] },
    Article { slug: "free-to-use", related: &["what-is", "need-account", "sources", "getting-started", "contribute"] },
    Article { slug: "wont-play", related: &["playback-unavailable", "black-screen", "buffering", "extensions-blockers", "supported-browsers"] },
    Article { slug: "playback-unavailable", related: &["wont-play", "sources", "extensions-blockers", "supported-browsers", "report-bug"] },
    Article { slug: "black-screen", related: &["wont-play", "supported-browsers", "extensions-blockers", "fix-phone", "fix-tv"] },
    Article { slug: "buffering", related: &["wont-play", "slow-pages", "supported-browsers", "extensions-blockers", "fix-tv"] },
    Article { slug: "subtitles-missing", related: &["subtitles", "language", "wont-play", "playback-unavailable", "report-bug"] },
    Article { slug: "supported-browsers", related: &["wont-play", "install-app", "fix-phone", "fix-tv", "extensions-blockers"] },
    Article { slug: "fix-phone", related: &["phone-tablet", "install-app", "supported-browsers", "wont-play", "buffering"] },
    Article { slug: "fix-tv", related: &["watch-on-tv", "wont-play", "supported-browsers", "buffering", "black-screen"] },
    Article { slug: "extensions-blockers", related: &["wont-play", "playback-unavailable", "supported-browsers", "black-screen", "sources"] },
    Article { slug: "rows-wont-load", related: &["supported-browsers", "report-bug", "recommendations", "extensions-blockers"] },
    Article { slug: "list-missing", related: &["my-list", "saved-data", "switch-profile", "continue-watching", "sign-in"] },
    Article { slug: "forgot-pin", related: &["profile-pin", "forgot-password", "profiles-manage", "kids-profile", "switch-profile"] },
    Article { slug: "language", related: &["subtitles", "browse-language", "subtitles-missing", "shortcuts", "accessibility"] },
    Article { slug: "subtitles", related: &["language", "subtitles-missing", "shortcuts", "accessibility", "browse-language"] },
    Article { slug: "shortcuts", related: &["accessibility", "skip-intro-next", "subtitles", "continue-watching", "wont-play"] },
    Article { slug: "skip-intro-next", related: &["shortcuts", "episodes", "continue-watching", "subtitles", "my-list"] },
    Article { slug: "my-list", related: &["list-missing", "continue-watching", "viewing-history", "saved-data", "recommendations"] },
    Article { slug: "continue-watching", related: &["viewing-history", "my-list", "saved-data", "clear-data", "skip-intro-next"] },
    Article { slug: "viewing-history", related: &["continue-watching", "clear-data", "saved-data", "recommendations", "privacy-data"] },
    Article { slug: "notifications", related: &["new-popular", "my-list", "recommendations", "trailers-clips", "search-browse"] },
    Article { slug: "search-browse", related: &["browse-language", "new-popular", "recommendations", "my-list", "trailers-clips"] },
    Article { slug: "browse-language", related: &["language", "subtitles", "search-browse", "new-popular", "recommendations"] },
    Article { slug: "new-popular", related: &["notifications", "search-browse", "trailers-clips", "recommendations", "my-list"] },
    Article { slug: "recommendations", related: &["viewing-history", "privacy-data", "my-list", "search-browse", "kids-profile"] },
    Article { slug: "trailers-clips", related: &["new-popular", "search-browse", "my-list", "recommendations", "notifications"] },
    Article { slug: "what-is", related: &["getting-started", "sources", "free-to-use", "need-account", "contribute"] },
    Article { slug: "getting-started", related: &["what-is", "profiles-manage", "search-browse", "install-app", "need-account"] },
    Article { slug: "sources", related: &["what-is", "wont-play", "playback-unavailable", "copyright", "free-to-use"] },
    Article { slug: "contribute", related: &["report-bug", "title-feedback", "what-is", "getting-started", "free-to-use"] },
    Article { slug: "install-app", related: &["phone-tablet", "supported-browsers", "fix-phone", "getting-started", "watch-on-tv"] },
    Article { slug: "phone-tablet", related: &["install-app", "fix-phone", "supported-browsers", "getting-started", "shortcuts"] },
    Article { slug: "watch-on-tv", related: &["fix-tv", "install-app", "supported-browsers", "getting-started", "shortcuts"] },
    Article { slug: "copyright", related: &["sources", "what-is", "privacy-data", "title-feedback", "report-bug"] },
    Article { slug: "title-feedback", related: &["report-bug", "contribute", "wont-play", "recommendations", "copyright"] },
    Article { slug: "report-bug", related: &["title-feedback", "contribute", "rows-wont-load", "wont-play"] },
    Article { slug: "fake-sites", related: &["free-to-use", "copyright", "security-report", "privacy-data", "what-is"] },
    Article { slug: "security-report", related: &["fake-sites", "report-bug", "privacy-data", "profile-pin", "contribute"] },
    Article { slug: "translate", related: &["contribute", "language", "report-bug", "what-is", "getting-started"] },
    Article { slug: "cookies-explained", related: &["privacy-data", "saved-data", "clear-data", "recommendations", "sources"] },
    Article { slug: "stuck-loading", related: &["supported-browsers", "extensions-blockers", "clear-data", "rows-wont-load"] },
    Article { slug: "slow-pages", related: &["buffering", "supported-browsers", "vpn-network", "trailers-clips", "wont-play"] },
    Article { slug: "vpn-network", related: &["is-down", "wont-play", "playback-unavailable", "rows-wont-load", "sources"] },
    Article { slug: "is-down", related: &["vpn-network", "wont-play", "rows-wont-load", "sources", "report-bug"] },
    Article { slug: "no-sound", related: &["autoplay-blocked", "shortcuts", "subtitles", "wont-play", "black-screen"] },
    Article { slug: "autoplay-blocked", related: &["no-sound", "trailers-clips", "wont-play", "shortcuts", "slow-pages"] },
    Article { slug: "clips", related: &["trailers-clips", "my-list", "new-popular", "search-browse", "recommendations"] },
    Article { slug: "genres", related: &["search-browse", "browse-language", "new-popular", "recommendations", "top-10"] },
    Article { slug: "top-10", related: &["new-popular", "genres", "recommendations", "search-browse", "trailers-clips"] },
    Article { slug: "episodes", related: &["skip-intro-next", "continue-watching", "shortcuts", "title-details", "my-list"] },
    Article { slug: "share", related: &["title-details", "my-list", "search-browse", "title-feedback", "new-popular"] },
    Article { slug: "title-details", related: &["my-list", "episodes", "share", "recommendations", "trailers-clips"] },
    Article { slug: "accessibility", related: &["subtitles", "shortcuts", "language", "phone-tablet", "report-bug"] },
    Article { slug: "profile-icons", related: &["profiles-manage", "kids-profile", "switch-profile", "profile-pin", "saved-data"] },
    Article { slug: "parental-controls", related: &["kids-profile", "profile-pin", "profiles-manage", "viewing-history", "switch-profile"] },
    Article { slug: "transfer-profile", related: &["profiles-manage", "download-data", "delete-account", "profile-pin", "saved-data"] },
    Article { slug: "forgot-password", related: &["sign-in", "change-email-password", "fake-sites", "devices-sign-out", "need-account"] },
    Article { slug: "change-email-password", related: &["forgot-password", "devices-sign-out", "sign-in", "delete-account", "privacy-data"] },
    Article { slug: "devices-sign-out", related: &["change-email-password", "sign-in", "forgot-password", "security-report", "privacy-data"] },
    Article { slug: "download-data", related: &["privacy-data", "delete-account", "viewing-history", "transfer-profile", "saved-data"] },
    Article { slug: "delete-account", related: &["download-data", "profiles-manage", "privacy-data", "transfer-profile", "clear-data"] },
];

pub fn article(slug: &str) -> Option<&'static Article> {
    ARTICLES.iter().find(|a| a.slug == slug)
}

/// Topic and group an article is listed under, used for the breadcrumb-free "Back to Help Home" and for search context.
pub fn locate(slug: &str) -> Option<(&'static Topic, &'static Group)> {
    TOPICS.iter().find_map(|t| t.groups.iter().find(|g| g.articles.contains(&slug)).map(|g| (t, g)))
}

pub fn key(slug: &str, part: &str) -> String {
    format!("help-art-{slug}-{part}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_listed_article_has_an_entry_and_every_link_resolves() {
        for t in TOPICS {
            for g in t.groups {
                for s in g.articles {
                    if let Some(id) = s.strip_prefix(DOC_PREFIX) {
                        assert!(legal_doc(id).is_some(), "{s} is listed but is not a policy");
                        continue;
                    }
                    assert!(article(s).is_some(), "{s} is listed but has no Article entry");
                }
            }
        }
        let ftl = include_str!("../../i18n/en.ftl");
        for d in LEGAL {
            assert!(ftl.contains(&format!("\nlegal-{}-title =", d.id)), "policy {} has no title", d.id);
            for n in 1..=d.sections {
                assert!(ftl.contains(&format!("\nlegal-{}-{}-title =", d.id, n)), "policy {} lacks section {}", d.id, n);
            }
            assert!(!ftl.contains(&format!("\nlegal-{}-{}-title =", d.id, d.sections + 1)), "policy {} has more sections than declared", d.id);
        }
        for a in ARTICLES {
            assert!(locate(a.slug).is_some(), "{} is not listed under any group", a.slug);
            assert!((3..=5).contains(&a.related.len()), "{} should have 3 to 5 related articles", a.slug);
            for r in a.related {
                assert!(article(r).is_some(), "{} links to missing {r}", a.slug);
                assert_ne!(*r, a.slug, "{} links to itself", a.slug);
            }
        }
        for p in POPULAR {
            assert!(article(p).is_some(), "popular {p} missing");
        }
    }
}
