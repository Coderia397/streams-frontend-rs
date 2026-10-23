//! One title in full: details, credits, logo, seasons and episodes.
use super::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TmdbImages {
    pub logos: Vec<TmdbImage>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TmdbImage {
    pub file_path: String,
    pub iso_639_1: Option<String>,
}

/// Title logos already looked up, kept in this browser for two weeks (title -> logo address, or none, and when it was learnt): a card
/// shows its logo at once on a later visit instead of asking TMDB for every title on the page (about 120 lookups on Home).
const LOGOS_KEY: &str = "streams_logos_v1";
const LOGOS_KEEP_MS: f64 = 14.0 * 24.0 * 3600.0 * 1000.0;
const LOGOS_MAX: usize = 3000;

type LogoMap = std::collections::HashMap<String, (Option<String>, f64)>;

thread_local! {
    static LOGOS: std::cell::RefCell<Option<LogoMap>> = const { std::cell::RefCell::new(None) };
    static LOGOS_SAVE_PENDING: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

fn with_logos<T>(f: impl FnOnce(&mut LogoMap) -> T) -> T {
    LOGOS.with(|l| {
        let mut l = l.borrow_mut();
        let map = l.get_or_insert_with(|| {
            let now = js_sys::Date::now();
            let mut map: LogoMap = web_sys::window()
                .and_then(|w| w.local_storage().ok().flatten())
                .and_then(|s| s.get_item(LOGOS_KEY).ok().flatten())
                .and_then(|raw| serde_json::from_str(&raw).ok())
                .unwrap_or_default();
            map.retain(|_, (_, at)| now - *at < LOGOS_KEEP_MS);
            map
        });
        f(map)
    })
}

/// Writes the logos to the browser a moment after the last one learnt (a page learns dozens at once; one write for them all).
fn save_logos_soon() {
    if LOGOS_SAVE_PENDING.with(|p| p.replace(true)) {
        return;
    }
    leptos::leptos_dom::helpers::set_timeout(
        || {
            LOGOS_SAVE_PENDING.with(|p| p.set(false));
            let raw = with_logos(|map| {
                if map.len() > LOGOS_MAX {
                    let mut by_age: Vec<(String, f64)> = map.iter().map(|(k, (_, at))| (k.clone(), *at)).collect();
                    by_age.sort_by(|a, b| a.1.total_cmp(&b.1));
                    for (k, _) in by_age.into_iter().take(map.len() - LOGOS_MAX) {
                        map.remove(&k);
                    }
                }
                serde_json::to_string(map).ok()
            });
            if let (Some(raw), Some(s)) = (raw, web_sys::window().and_then(|w| w.local_storage().ok().flatten())) {
                let _ = s.set_item(LOGOS_KEY, &raw);
            }
        },
        std::time::Duration::from_millis(1500),
    );
}

pub async fn fetch_movie_logo(id: u32, is_tv: bool) -> Result<Option<String>, gloo_net::Error> {
    let kind = if is_tv { "tv" } else { "movie" };
    let key = format!("{kind}:{id}");
    if let Some((logo, _)) = with_logos(|map| map.get(&key).cloned()) {
        return Ok(logo);
    }
    let logo = fetch_logo_from_tmdb(kind, id).await?;
    with_logos(|map| map.insert(key, (logo.clone(), js_sys::Date::now())));
    save_logos_soon();
    Ok(logo)
}

/// The English logo, else one without text, else any. TMDB's answer carries every poster and backdrop too (26 KB a title on average,
/// 183 titles on a cold Home), so it is asked for English and language-free pictures first, which holds a logo for 177 of 183
/// titles at about half the size; the full list only when that one has no logo (measured 2026-10-23, note 115).
async fn fetch_logo_from_tmdb(kind: &str, id: u32) -> Result<Option<String>, gloo_net::Error> {
    let url = format!("{}/{}/{}/images?api_key={}", TMDB_BASE_URL, kind, id, TMDB_API_KEY);
    let mut resp: TmdbImages = get_json(&format!("{url}&include_image_language=en,null")).await?;
    if resp.logos.is_empty() {
        resp = get_json(&url).await?;
    }
    let logo = resp.logos.iter().find(|l| l.iso_639_1.as_deref() == Some("en"))
        .or_else(|| resp.logos.iter().find(|l| l.iso_639_1.is_none()))
        .or_else(|| resp.logos.first());
    Ok(logo.map(|l| format!("{}/w300{}", TMDB_IMAGE_BASE, l.file_path)))
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct DetailedMediaItem {
    pub id: u32,
    pub title: Option<String>,
    pub name: Option<String>,
    pub overview: String,
    pub poster_path: Option<String>,
    pub backdrop_path: Option<String>,
    pub vote_average: f64,
    pub release_date: Option<String>,
    pub first_air_date: Option<String>,
    pub runtime: Option<u32>,
    pub number_of_seasons: Option<u32>,
    pub credits: Option<Credits>,
    pub genres: Option<Vec<Genre>>,
    #[serde(default)]
    pub original_language: Option<String>,
    /// the people who created a series (TMDB gives none for films)
    #[serde(default)]
    pub created_by: Option<Vec<CastMember>>,
    /// where a series first aired (Netflix, HBO ...), with their logos
    #[serde(default)]
    pub networks: Option<Vec<Company>>,
    /// the companies that made a film or series, with their logos
    #[serde(default)]
    pub production_companies: Option<Vec<Company>>,
}

#[derive(Clone, Debug, Default, serde::Deserialize, serde::Serialize)]
pub struct Company {
    pub name: String,
    #[serde(default)]
    pub logo_path: Option<String>,
}

impl DetailedMediaItem {
    pub fn display_title(&self) -> &str {
        self.title.as_deref().or(self.name.as_deref()).unwrap_or("Unknown")
    }
}

#[derive(Clone, Debug, Default, serde::Deserialize, serde::Serialize)]
pub struct Credits {
    #[serde(default)]
    pub cast: Vec<CastMember>,
    #[serde(default)]
    pub crew: Vec<CrewMember>,
}

#[derive(Clone, Debug, Default, serde::Deserialize, serde::Serialize)]
pub struct CastMember {
    pub name: String,
}

#[derive(Clone, Debug, Default, serde::Deserialize, serde::Serialize)]
pub struct CrewMember {
    pub name: String,
    pub job: Option<String>,
    pub department: Option<String>,
}

pub async fn fetch_details(id: u32, is_tv: bool) -> Result<DetailedMediaItem, gloo_net::Error> {
    let kind = if is_tv { "tv" } else { "movie" };
    let url = format!(
        "{}/{}/{}?api_key={}&append_to_response=credits",
        TMDB_BASE_URL, kind, id, TMDB_API_KEY
    );
    get_json(&url).await
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct Genre {
    pub id: u32,
    pub name: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SeasonDetails {
    pub _id: String,
    pub episodes: Vec<Episode>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Episode {
    pub id: u32,
    pub name: String,
    pub overview: String,
    pub episode_number: u32,
    pub still_path: Option<String>,
    pub runtime: Option<u32>,
}

impl Episode {
    pub fn still_url(&self, size: &str) -> Option<String> {
        self.still_path
            .as_ref()
            .map(|p| format!("{}/{}{}", TMDB_IMAGE_BASE, size, p))
    }
}

/// Fetch TV season episodes
pub async fn fetch_season_details(series_id: u32, season_number: u32) -> Result<SeasonDetails, gloo_net::Error> {
    let url = format!(
        "{}/tv/{}/season/{}?api_key={}&language=en-US",
        TMDB_BASE_URL, series_id, season_number, TMDB_API_KEY
    );
    get_json(&url).await
}
