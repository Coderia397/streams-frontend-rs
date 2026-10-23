//! The title as lists and rows carry it (MediaItem), with its pictures and its hooks.
use super::*;

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct MediaItem {
    pub id: u32,
    pub title: Option<String>,
    pub name: Option<String>,
    #[serde(default, deserialize_with = "deserialize_null_string")]
    pub overview: String,
    pub poster_path: Option<String>,
    pub backdrop_path: Option<String>,
    #[serde(default, deserialize_with = "deserialize_null_f64")]
    pub vote_average: f64,
    #[serde(default)]
    pub vote_count: Option<u32>,
    #[serde(default)]
    pub adult: Option<bool>,
    pub release_date: Option<String>,
    pub first_air_date: Option<String>,
    pub media_type: Option<String>,
    #[serde(default)]
    pub match_percentage: Option<u32>,
    #[serde(default)]
    pub genre_ids: Vec<u32>,
    #[serde(default)]
    pub vibe_pills: Vec<String>,
    /// The real British and American certificates when the engine sent them
    #[serde(default)]
    pub rating_gb: Option<String>,
    #[serde(default)]
    pub rating_us: Option<String>,
    /// Labels from the engine (`new`, `top10`)
    #[serde(default)]
    pub badges: Vec<String>,
    /// The lines of the hero banner the engine chose for this title
    #[serde(default)]
    pub hooks: Vec<Hook>,
}

/// One line of the hero banner: what kind it is (for its icon colour), the icon, the translation key with its arguments, and the English text.
#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Hook {
    pub kind: String,
    pub icon: String,
    pub key: String,
    #[serde(default)]
    pub args: serde_json::Value,
    pub text: String,
}

impl MediaItem {
    pub fn display_title(&self) -> &str {
        self.title
            .as_deref()
            .or(self.name.as_deref())
            .unwrap_or("Unknown Title")
    }

    pub fn is_nsfw(&self) -> bool {
        if self.adult.unwrap_or(false) {
            return true;
        }
        let t = self.display_title().to_lowercase();
        const BLOCKED: &[&str] = &[
            "overflow", "おーばーふろぉ", "じ〜くれっとみっしょん", "secret mission",
            "sweet agony", "hentai", "ecchi", "erotic", "porn", "xxx",
        ];
        for &term in BLOCKED {
            if t.contains(term) {
                return true;
            }
        }
        false
    }

    /// Full poster URL at a given size (e.g. "w500", "original")
    pub fn poster_url(&self, size: &str) -> Option<String> {
        self.poster_path
            .as_ref()
            .filter(|p| !p.is_empty())
            .map(|p| format!("{}/{}{}", TMDB_IMAGE_BASE, size, p))
    }

    /// Full backdrop URL at a given size
    pub fn backdrop_url(&self, size: &str) -> Option<String> {
        self.backdrop_path
            .as_ref()
            .filter(|p| !p.is_empty()) // the engine sends "" for a title with no picture: no address, not a broken one
            .map(|p| format!("{}/{}{}", TMDB_IMAGE_BASE, size, p))
    }

    pub fn is_tv(&self) -> bool {
        self.media_type.as_deref() == Some("tv")
            || self.name.is_some() && self.title.is_none()
    }
}

impl TmdbResponse<MediaItem> {
    /// Drops the titles the profile in use may not see (its age rating limit), see `services::rating`.
    pub async fn rated(mut self) -> Self {
        self.results = crate::services::rating::filter_allowed(std::mem::take(&mut self.results)).await;
        self
    }
}
