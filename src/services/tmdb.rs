use gloo_net::http::Request;
use serde::{Deserialize, Serialize};
use crate::models::mapping::{map_genre_key_to_tmdb, MediaKind};

const TMDB_BASE_URL: &str = crate::config::TMDB_API_URL;

const TMDB_API_KEY: &str = crate::config::TMDB_API_KEY;
const TMDB_IMAGE_BASE: &str = crate::config::TMDB_IMAGE_URL;

/// Adds the interface language to a catalog request so titles, summaries and genre names come back translated. Trailers and
/// logos are left alone because their language filters would hide most of them.
fn localised(url: &str) -> String {
    if url.contains("/videos") || url.contains("/images") {
        return url.to_string();
    }
    let lang = crate::i18n::tmdb_language();
    if url.contains("language=") {
        url.replace("language=en-US", &format!("language={lang}"))
    } else if url.contains('?') {
        format!("{url}&language={lang}")
    } else {
        format!("{url}?language={lang}")
    }
}

fn get(url: &str) -> gloo_net::http::RequestBuilder {
    Request::get(&localised(url))
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TmdbResponse<T> {
    pub page: u32,
    pub results: Vec<T>,
    pub total_pages: u32,
    pub total_results: u32,
}

fn deserialize_null_string<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let opt = Option::<String>::deserialize(deserializer)?;
    Ok(opt.unwrap_or_default())
}

fn deserialize_null_f64<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let opt = Option::<f64>::deserialize(deserializer)?;
    Ok(opt.unwrap_or(0.0))
}

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

/// Fetch trending items (used for the hero carousel and home page rows).
pub async fn fetch_trending(kind: &str) -> Result<Vec<MediaItem>, gloo_net::Error> {
    let url = format!(
        "{}/trending/{}/week?api_key={}&language=en-US",
        TMDB_BASE_URL, kind, TMDB_API_KEY
    );
    let resp: TmdbResponse<MediaItem> = get(&url).send().await?.json::<TmdbResponse<MediaItem>>().await?.rated().await;
    let items = resp.results.into_iter().filter(|i| !i.is_nsfw() && (i.backdrop_path.is_some() || i.poster_path.is_some())).collect();
    Ok(items)
}

/// Fetch top rated items
pub async fn fetch_top_rated(kind: &str) -> Result<Vec<MediaItem>, gloo_net::Error> {
    let url = format!(
        "{}/{}/top_rated?api_key={}&language=en-US",
        TMDB_BASE_URL, kind, TMDB_API_KEY
    );
    let resp: TmdbResponse<MediaItem> = get(&url).send().await?.json::<TmdbResponse<MediaItem>>().await?.rated().await;
    let items = resp.results.into_iter().filter(|i| !i.is_nsfw() && (i.backdrop_path.is_some() || i.poster_path.is_some())).collect();
    Ok(items)
}

/// Fetch items for a specific TMDB genre ID.
pub async fn fetch_by_genre(kind: &str, genre_id: u32) -> Result<Vec<MediaItem>, gloo_net::Error> {
    let url = format!(
        "{}/discover/{}?api_key={}&with_genres={}&language=en-US&sort_by=popularity.desc",
        TMDB_BASE_URL, kind, TMDB_API_KEY, genre_id
    );
    let resp: TmdbResponse<MediaItem> = get(&url).send().await?.json::<TmdbResponse<MediaItem>>().await?.rated().await;
    Ok(resp.results)
}

/// Fetch items for a British region filter (used by the 'British TV' category).
pub async fn fetch_british_tv() -> Result<Vec<MediaItem>, gloo_net::Error> {
    let url = format!(
        "{}/discover/tv?api_key={}&with_origin_country=GB&language=en-US&sort_by=popularity.desc",
        TMDB_BASE_URL, TMDB_API_KEY
    );
    let resp: TmdbResponse<MediaItem> = get(&url).send().await?.json::<TmdbResponse<MediaItem>>().await?.rated().await;
    Ok(resp.results)
}

/// Primary entry point: given a Netflix-style genre ID from the URL,
/// resolve the correct TMDB query and return its results.
pub async fn fetch_for_genre_route(genre_key: &str) -> Result<Vec<MediaItem>, gloo_net::Error> {
    let kind = if let Some(ctx) = map_genre_key_to_tmdb(genre_key) {
        match ctx.kind {
            MediaKind::Tv => "tv",
            MediaKind::Movie => "movie",
        }
    } else if genre_key == "british" || genre_key == "binge" || genre_key == "52117" || genre_key == "1191605" {
        "tv"
    } else {
        "movie"
    };
    fetch_row_content(kind, Some(genre_key), None, None, None, 1).await
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TmdbImages {
    pub logos: Vec<TmdbImage>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TmdbImage {
    pub file_path: String,
    pub iso_639_1: Option<String>,
}

pub async fn fetch_movie_logo(id: u32, is_tv: bool) -> Result<Option<String>, gloo_net::Error> {
    let kind = if is_tv { "tv" } else { "movie" };
    let url = format!(
        "{}/{}/{}/images?api_key={}",
        TMDB_BASE_URL, kind, id, TMDB_API_KEY
    );
    let resp: TmdbImages = get(&url).send().await?.json().await?;
    
    // Find english logo or first available
    let logo = resp.logos.iter().find(|l| l.iso_639_1.as_deref() == Some("en"))
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
    get(&url).send().await?.json().await
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct Genre {
    pub id: u32,
    pub name: String,
}

/// Search across TV and Movies
pub async fn search_media(query: &str) -> Result<Vec<MediaItem>, gloo_net::Error> {
    let clean = query.trim();
    if clean.is_empty() {
        return Ok(vec![]);
    }
    let encoded_query: String = js_sys::encode_uri_component(clean)
        .as_string()
        .unwrap_or_else(|| clean.to_string());
    let url = format!(
        "{}/search/multi?api_key={}&language=en-US&query={}&page=1&include_adult=false",
        TMDB_BASE_URL, TMDB_API_KEY, encoded_query
    );
    let resp: TmdbResponse<MediaItem> = get(&url).send().await?.json::<TmdbResponse<MediaItem>>().await?.rated().await;
    // Filter out people, nsfw content, and items without artwork
    let filtered = resp.results.into_iter().filter(|i| {
        i.media_type.as_deref() != Some("person")
            && !i.is_nsfw()
            && (i.backdrop_path.is_some() || i.poster_path.is_some())
    }).collect();
    Ok(filtered)
}

/// Fetch recommendations (More Like This)
pub async fn fetch_recommendations(id: u32, is_tv: bool) -> Result<Vec<MediaItem>, gloo_net::Error> {
    let kind = if is_tv { "tv" } else { "movie" };
    let url = format!(
        "{}/{}/{}/recommendations?api_key={}&language=en-US&page=1",
        TMDB_BASE_URL, kind, id, TMDB_API_KEY
    );
    let resp: TmdbResponse<MediaItem> = get(&url).send().await?.json::<TmdbResponse<MediaItem>>().await?.rated().await;
    let filtered = resp.results.into_iter().filter(|i| {
        !i.is_nsfw() && (i.backdrop_path.is_some() || i.poster_path.is_some())
    }).collect();
    Ok(filtered)
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct VideoResult {
    pub key: String,
    pub site: String,
    pub r#type: String, // "Trailer", "Clip", etc
    #[serde(default)]
    pub official: Option<bool>,
    #[serde(default)]
    pub name: Option<String>,
}

/// Smart scored selector for TMDB videos:
/// Evaluates official verification, trailer vs teaser classification,
/// resolution signals (4K/UHD), and eliminates junk (interviews, featurettes, B-roll).
/// Returns (video_key, is_teaser).
pub fn select_best_tmdb_trailer(videos: &[VideoResult]) -> Option<(String, bool)> {
    let yt_videos: Vec<&VideoResult> = videos
        .iter()
        .filter(|v| v.site.eq_ignore_ascii_case("youtube") && !v.key.trim().is_empty())
        .collect();

    if yt_videos.is_empty() {
        return None;
    }

    let mut scored: Vec<(&VideoResult, i32, bool)> = yt_videos
        .into_iter()
        .map(|v| {
            let mut score = 0i32;
            let mut is_teaser = false;
            let type_lower = v.r#type.to_lowercase();
            let name_lower = v.name.as_deref().unwrap_or("").to_lowercase();

            // 1. Type scoring
            if type_lower == "trailer" {
                score += 100;
            } else if type_lower == "teaser" {
                score += 80;
                is_teaser = true;
            } else if type_lower == "clip" {
                score += 30;
                is_teaser = true;
            } else {
                score -= 80; // Featurette, Behind the Scenes, etc.
            }

            // 2. Official badge bonus
            if v.official == Some(true) {
                score += 50;
            }

            // 3. Name intelligence
            if name_lower.contains("teaser") {
                is_teaser = true;
                score += 15;
            }
            if name_lower.contains("official") {
                score += 25;
            }
            if name_lower.contains("main") || name_lower.contains("final") {
                score += 35;
            }
            if name_lower.contains("4k") || name_lower.contains("uhd") {
                score += 25;
            }
            if name_lower.contains("imax") {
                score += 15;
            }

            // 4. Anti-junk filters
            let junk = [
                "behind the scenes", "interview", "b-roll", "featurette",
                "making of", "cast", "red carpet", "blooper", "soundtrack",
                "theme song", "clip: ", "scene: ", "deleted scene"
            ];
            for j in junk {
                if name_lower.contains(j) {
                    score -= 200;
                    break;
                }
            }

            (v, score, is_teaser)
        })
        .collect();

    scored.sort_by(|a, b| b.1.cmp(&a.1));

    scored.first().map(|(v, _, is_teaser)| (v.key.clone(), *is_teaser))
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TmdbVideos {
    pub results: Vec<VideoResult>,
}

/// Fetch YouTube trailers/clips
pub async fn fetch_videos(id: u32, is_tv: bool) -> Result<Vec<VideoResult>, gloo_net::Error> {
    let kind = if is_tv { "tv" } else { "movie" };
    let url = format!(
        "{}/{}/{}/videos?api_key={}&language=en-US",
        TMDB_BASE_URL, kind, id, TMDB_API_KEY
    );
    let resp: TmdbVideos = get(&url).send().await?.json().await?;
    Ok(resp.results)
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
    get(&url).send().await?.json().await
}

pub async fn fetch_trending_page(kind: &str, page: u32) -> Result<Vec<MediaItem>, gloo_net::Error> {
    let safe_kind = if kind == "tv" { "tv" } else { "movie" };
    let url = format!("{}/trending/{}/day?api_key={}&page={}", TMDB_BASE_URL, safe_kind, TMDB_API_KEY, page);
    let resp = get(&url).send().await?;
    let data: TmdbResponse<MediaItem> = resp.json::<TmdbResponse<MediaItem>>().await?.rated().await;
    let items = data.results.into_iter().filter_map(|mut item| {
        if item.is_nsfw() {
            return None;
        }
        if item.backdrop_path.is_none() && item.poster_path.is_none() {
            return None;
        }
        if item.media_type.is_none() {
            item.media_type = Some(safe_kind.to_string());
        }
        Some(item)
    }).collect();
    Ok(items)
}

/// Robust TMDB Query Engine for Rows, Feeds, and Category Pages.
/// Dispatches to distinct endpoints and queries with correct media_type separation,
/// quality floors, content moderation, and anti-leakage filters.
pub async fn fetch_row_content(
    kind: &str,
    genre_id: Option<&str>,
    sort_by: Option<&str>,
    extra_params: Option<&str>,
    endpoint: Option<&str>,
    page: u32,
) -> Result<Vec<MediaItem>, gloo_net::Error> {
    let safe_kind = if kind == "tv" { "tv" } else { "movie" };

    // 1. Explicit Endpoint (e.g. "/movie/now_playing", "/movie/upcoming", "/trending/tv/week")
    if let Some(ep) = endpoint {
        let ep_clean = ep.trim_start_matches('/');
        let sep = if ep_clean.contains('?') { "&" } else { "?" };
        let mut url = format!("{}/{}{}api_key={}&include_adult=false&page={}", TMDB_BASE_URL, ep_clean, sep, TMDB_API_KEY, page);
        if let Some(extra) = extra_params {
            if !extra.is_empty() {
                let extra_clean = extra.trim_start_matches('&');
                url.push_str(&format!("&{}", extra_clean));
            }
        }
        if let Ok(resp) = get(&url).send().await {
            if let Ok(data) = resp.json::<TmdbResponse<MediaItem>>().await {
                let data = data.rated().await;
                let items: Vec<MediaItem> = data.results.into_iter().filter_map(|mut item| {
                    if item.is_nsfw() {
                        return None;
                    }
                    if item.backdrop_path.is_none() && item.poster_path.is_none() {
                        return None;
                    }
                    if item.media_type.is_none() {
                        item.media_type = Some(safe_kind.to_string());
                    }
                    Some(item)
                }).collect();
                if !items.is_empty() {
                    return Ok(items);
                }
            }
        }
        return fetch_trending_page(safe_kind, page).await;
    }

    // 2. Genre or Category ID
    if let Some(gid) = genre_id {
        let gid_lower = gid.to_lowercase();
        let default_sort = sort_by.unwrap_or("popularity.desc");
        let has_custom_votes = extra_params.map(|e| e.contains("vote_count.gte")).unwrap_or(false);

        let mut url = match gid_lower.as_str() {
            // LGBTQ / Pride
            "10001" | "lgbtq" | "pride" => {
                let vote_filter = if has_custom_votes { "" } else { "&vote_count.gte=15" };
                format!("{}/discover/{}?api_key={}&with_keywords=158718%7C380747%7C250606%7C363345%7C264386%7C290527%7C329968%7C300642%7C195624{}&include_adult=false&sort_by={}&page={}", TMDB_BASE_URL, safe_kind, TMDB_API_KEY, vote_filter, default_sort, page)
            }
            // Black Stories
            "10003" | "black stories" | "black" => {
                let vote_filter = if has_custom_votes { "" } else { "&vote_count.gte=15" };
                format!("{}/discover/{}?api_key={}&with_keywords=256015%7C358563%7C190675%7C251436%7C288242%7C195624%7C339021{}&include_adult=false&sort_by={}&page={}", TMDB_BASE_URL, safe_kind, TMDB_API_KEY, vote_filter, default_sort, page)
            }
            // British origin
            "british" | "52117" | "10005" => {
                let vote_filter = if has_custom_votes { "" } else { "&vote_count.gte=25" };
                format!("{}/discover/{}?api_key={}&with_origin_country=GB{}&include_adult=false&sort_by={}&page={}", TMDB_BASE_URL, safe_kind, TMDB_API_KEY, vote_filter, default_sort, page)
            }
            // Classics (pre-1985 for movies, pre-1998 for TV)
            "10009" | "classics" => {
                if safe_kind == "tv" {
                    let vote_filter = if has_custom_votes { "" } else { "&vote_count.gte=40" };
                    format!("{}/discover/tv?api_key={}&first_air_date.lte=1998-12-31{}&include_adult=false&sort_by={}&page={}", TMDB_BASE_URL, TMDB_API_KEY, vote_filter, default_sort, page)
                } else {
                    let vote_filter = if has_custom_votes { "" } else { "&vote_count.gte=80" };
                    format!("{}/discover/movie?api_key={}&primary_release_date.lte=1985-12-31{}&include_adult=false&sort_by={}&page={}", TMDB_BASE_URL, TMDB_API_KEY, vote_filter, default_sort, page)
                }
            }
            // Cult Classics & Cult TV
            "10010" | "cult" => {
                let vote_filter = if has_custom_votes { "" } else { "&vote_count.gte=25" };
                format!("{}/discover/{}?api_key={}&with_keywords=374649%7C367333%7C9840{}&include_adult=false&sort_by={}&page={}", TMDB_BASE_URL, safe_kind, TMDB_API_KEY, vote_filter, default_sort, page)
            }
            // European origin
            "european" | "10006" => {
                let vote_filter = if has_custom_votes { "" } else { "&vote_count.gte=25" };
                format!("{}/discover/{}?api_key={}&with_origin_country=FR%7CDE%7CIT%7CES%7CNL%7CDK%7CSE%7CNO%7CFI%7CPL{}&include_adult=false&sort_by={}&page={}", TMDB_BASE_URL, safe_kind, TMDB_API_KEY, vote_filter, default_sort, page)
            }
            // US / Hollywood origin
            "us" | "10008" | "hollywood" => {
                let vote_filter = if has_custom_votes { "" } else { "&vote_count.gte=40" };
                format!("{}/discover/{}?api_key={}&with_origin_country=US{}&include_adult=false&sort_by={}&page={}", TMDB_BASE_URL, safe_kind, TMDB_API_KEY, vote_filter, default_sort, page)
            }
            // Halloween (Spooky & Supernatural)
            "10016" | "halloween" => {
                if safe_kind == "tv" {
                    let vote_filter = if has_custom_votes { "" } else { "&vote_count.gte=20" };
                    format!("{}/discover/tv?api_key={}&with_keywords=3335%7C3358%7C616%7C162846%7C3133%7C12339%7C1299%7C6152{}&include_adult=false&sort_by={}&page={}", TMDB_BASE_URL, TMDB_API_KEY, vote_filter, default_sort, page)
                } else {
                    let vote_filter = if has_custom_votes { "" } else { "&vote_count.gte=30" };
                    format!("{}/discover/movie?api_key={}&with_genres=27&with_keywords=3335%7C3358%7C616%7C162846%7C3133%7C12339%7C1299%7C6152{}&include_adult=false&sort_by={}&page={}", TMDB_BASE_URL, TMDB_API_KEY, vote_filter, default_sort, page)
                }
            }
            // Independent / Indie Cinema
            "10011" | "independent" | "indie" => {
                let vote_filter = if has_custom_votes { "" } else { "&vote_count.gte=20" };
                format!("{}/discover/{}?api_key={}&with_keywords=281237%7C10183{}&include_adult=false&sort_by={}&page={}", TMDB_BASE_URL, safe_kind, TMDB_API_KEY, vote_filter, default_sort, page)
            }
            // International (non-English)
            "international" | "10012" => {
                let vote_filter = if has_custom_votes { "" } else { "&vote_count.gte=30" };
                format!("{}/discover/{}?api_key={}&without_original_language=en{}&include_adult=false&sort_by={}&page={}", TMDB_BASE_URL, safe_kind, TMDB_API_KEY, vote_filter, default_sort, page)
            }
            // Shorts (under 45 minutes)
            "10013" | "shorts" => {
                let vote_filter = if has_custom_votes { "" } else { "&vote_count.gte=10" };
                format!("{}/discover/movie?api_key={}&with_runtime.lte=40&with_runtime.gte=2{}&include_adult=false&sort_by={}&page={}", TMDB_BASE_URL, TMDB_API_KEY, vote_filter, default_sort, page)
            }
            // Sports
            "10014" | "sport" | "sports" => {
                let vote_filter = if has_custom_votes { "" } else { "&vote_count.gte=20" };
                format!("{}/discover/{}?api_key={}&with_keywords=333328%7C13042%7C6496%7C1480%7C209476%7C315138{}&include_adult=false&sort_by={}&page={}", TMDB_BASE_URL, safe_kind, TMDB_API_KEY, vote_filter, default_sort, page)
            }
            // Stand-up Comedy (Movies/Specials)
            "10017" | "stand-up comedy" | "stand-up" => {
                let vote_filter = if has_custom_votes { "" } else { "&vote_count.gte=15" };
                format!("{}/discover/movie?api_key={}&with_genres=35&with_keywords=9716%7C356038{}&include_adult=false&sort_by={}&page={}", TMDB_BASE_URL, TMDB_API_KEY, vote_filter, default_sort, page)
            }
            // Stand-up & Chat Shows (TV)
            "10020" | "chat shows" | "stand-up & chat shows" => {
                let vote_filter = if has_custom_votes { "" } else { "&vote_count.gte=10" };
                format!("{}/discover/tv?api_key={}&with_genres=10767&with_keywords=9716%7C356038{}&include_adult=false&sort_by={}&page={}", TMDB_BASE_URL, TMDB_API_KEY, vote_filter, default_sort, page)
            }
            // Emmy or Emmys (Award-winning prestige television)
            "10018" | "emmy" | "emmys" | "emmy or emmys" => {
                let vote_filter = if has_custom_votes { "" } else { "&vote_count.gte=80" };
                format!("{}/discover/tv?api_key={}&with_genres=18,35&vote_average.gte=7.8{}&include_adult=false&sort_by={}&page={}", TMDB_BASE_URL, TMDB_API_KEY, vote_filter, default_sort, page)
            }
            // Science & Nature
            "10019" | "science & nature" | "nature" => {
                let vote_filter = if has_custom_votes { "" } else { "&vote_count.gte=15" };
                if safe_kind == "tv" {
                    format!("{}/discover/tv?api_key={}&with_genres=99&with_keywords=221355%7C9902%7C270%7C33577%7C305903{}&include_adult=false&sort_by={}&page={}", TMDB_BASE_URL, TMDB_API_KEY, vote_filter, default_sort, page)
                } else {
                    format!("{}/discover/movie?api_key={}&with_genres=99&with_keywords=221355%7C9902%7C270%7C33577%7C305903{}&include_adult=false&sort_by={}&page={}", TMDB_BASE_URL, TMDB_API_KEY, vote_filter, default_sort, page)
                }
            }
            // Teen
            "10015" | "teen" => {
                let vote_filter = if has_custom_votes { "" } else { "&vote_count.gte=25" };
                format!("{}/discover/{}?api_key={}&with_genres=18,35&with_keywords=10683%7C6270%7C193400%7C206720{}&include_adult=false&sort_by={}&page={}", TMDB_BASE_URL, safe_kind, TMDB_API_KEY, vote_filter, default_sort, page)
            }
            // Anime (Strict vote floor + Japanese language + exclude adult tags)
            "anime" | "7424" | "16" => {
                let vote_filter = if has_custom_votes { "" } else { "&vote_count.gte=40" };
                format!("{}/discover/{}?api_key={}&with_genres=16&with_original_language=ja{}&without_keywords=190370,222243,267498&include_adult=false&sort_by={}&page={}", TMDB_BASE_URL, safe_kind, TMDB_API_KEY, vote_filter, default_sort, page)
            }
            // Horror (Cross-media: TV uses Mystery/Sci-Fi + Horror keywords)
            "27" | "8711" | "horror" => {
                if safe_kind == "tv" {
                    let vote_filter = if has_custom_votes { "" } else { "&vote_count.gte=25" };
                    format!("{}/discover/tv?api_key={}&with_keywords=6152%7C3358%7C12377%7C162846%7C295907%7C3335{}&include_adult=false&sort_by={}&page={}", TMDB_BASE_URL, TMDB_API_KEY, vote_filter, default_sort, page)
                } else {
                    let vote_filter = if has_custom_votes { "" } else { "&vote_count.gte=35" };
                    format!("{}/discover/movie?api_key={}&with_genres=27{}&include_adult=false&sort_by={}&page={}", TMDB_BASE_URL, TMDB_API_KEY, vote_filter, default_sort, page)
                }
            }
            // Romance (Cross-media: TV uses Drama + Romance keywords)
            "10749" | "8883" | "romance" => {
                if safe_kind == "tv" {
                    let vote_filter = if has_custom_votes { "" } else { "&vote_count.gte=25" };
                    format!("{}/discover/tv?api_key={}&with_genres=18&with_keywords=304976%7C128%7C3691%7C13072{}&include_adult=false&sort_by={}&page={}", TMDB_BASE_URL, TMDB_API_KEY, vote_filter, default_sort, page)
                } else {
                    let vote_filter = if has_custom_votes { "" } else { "&vote_count.gte=35" };
                    format!("{}/discover/movie?api_key={}&with_genres=10749{}&include_adult=false&sort_by={}&page={}", TMDB_BASE_URL, TMDB_API_KEY, vote_filter, default_sort, page)
                }
            }
            // Thriller (Cross-media: TV uses Mystery & Crime)
            "53" | "8933" | "thriller" => {
                if safe_kind == "tv" {
                    let vote_filter = if has_custom_votes { "" } else { "&vote_count.gte=30" };
                    format!("{}/discover/tv?api_key={}&with_genres=9648,80{}&include_adult=false&sort_by={}&page={}", TMDB_BASE_URL, TMDB_API_KEY, vote_filter, default_sort, page)
                } else {
                    let vote_filter = if has_custom_votes { "" } else { "&vote_count.gte=35" };
                    format!("{}/discover/movie?api_key={}&with_genres=53{}&include_adult=false&sort_by={}&page={}", TMDB_BASE_URL, TMDB_API_KEY, vote_filter, default_sort, page)
                }
            }
            // Fantasy (Cross-media: TV uses 10765)
            "14" | "fantasy" => {
                if safe_kind == "tv" {
                    let vote_filter = if has_custom_votes { "" } else { "&vote_count.gte=25" };
                    format!("{}/discover/tv?api_key={}&with_genres=10765&without_genres=16{}&include_adult=false&sort_by={}&page={}", TMDB_BASE_URL, TMDB_API_KEY, vote_filter, default_sort, page)
                } else {
                    let vote_filter = if has_custom_votes { "" } else { "&vote_count.gte=35" };
                    format!("{}/discover/movie?api_key={}&with_genres=14{}&include_adult=false&sort_by={}&page={}", TMDB_BASE_URL, TMDB_API_KEY, vote_filter, default_sort, page)
                }
            }
            // Sci-Fi (Cross-media: TV uses 10765)
            "878" | "1492" | "1372" | "sci-fi" => {
                if safe_kind == "tv" {
                    let vote_filter = if has_custom_votes { "" } else { "&vote_count.gte=30" };
                    format!("{}/discover/tv?api_key={}&with_genres=10765&without_genres=16{}&include_adult=false&sort_by={}&page={}", TMDB_BASE_URL, TMDB_API_KEY, vote_filter, default_sort, page)
                } else {
                    let vote_filter = if has_custom_votes { "" } else { "&vote_count.gte=35" };
                    format!("{}/discover/movie?api_key={}&with_genres=878{}&include_adult=false&sort_by={}&page={}", TMDB_BASE_URL, TMDB_API_KEY, vote_filter, default_sort, page)
                }
            }
            // Music & Musicals
            "10402" | "music & musicals" => {
                if safe_kind == "tv" {
                    let vote_filter = if has_custom_votes { "" } else { "&vote_count.gte=15" };
                    format!("{}/discover/tv?api_key={}&with_keywords=4344%7C246377%7C6029%7C18001{}&include_adult=false&sort_by={}&page={}", TMDB_BASE_URL, TMDB_API_KEY, vote_filter, default_sort, page)
                } else {
                    let vote_filter = if has_custom_votes { "" } else { "&vote_count.gte=25" };
                    format!("{}/discover/movie?api_key={}&with_genres=10402{}&include_adult=false&sort_by={}&page={}", TMDB_BASE_URL, TMDB_API_KEY, vote_filter, default_sort, page)
                }
            }
            // Binge-worthy / Boredom Busters
            "binge" | "1191605" | "boredom" => {
                let vote_filter = if has_custom_votes { "" } else { "&vote_count.gte=40" };
                if safe_kind == "tv" {
                    format!("{}/discover/tv?api_key={}&with_genres=10759,80,10765&without_genres=16,10764{}&include_adult=false&sort_by={}&page={}", TMDB_BASE_URL, TMDB_API_KEY, vote_filter, default_sort, page)
                } else {
                    format!("{}/discover/movie?api_key={}&with_genres=28,12,53,878&without_genres=16{}&include_adult=false&sort_by={}&page={}", TMDB_BASE_URL, TMDB_API_KEY, vote_filter, default_sort, page)
                }
            }
            // Numeric IDs or Netflix codes
            _ => {
                let resolved_genre_id = match gid_lower.as_str() {
                    "1365" => if safe_kind == "tv" { "10759" } else { "28" },
                    "6548" | "10375" => "35",
                    "5763" | "11714" => "18",
                    "6839" => "99",
                    "9875" | "26146" => "80",
                    "783" => if safe_kind == "tv" { "10762" } else { "10751" },
                    "10673" => "10759",
                    // TMDB cross-media resolution
                    "28" | "12" => if safe_kind == "tv" { "10759" } else { gid },
                    "10751" => if safe_kind == "tv" { "10762" } else { "10751" },
                    "10759" => if safe_kind == "movie" { "28" } else { "10759" },
                    "10765" => if safe_kind == "movie" { "878" } else { "10765" },
                    "10762" => if safe_kind == "movie" { "10751" } else { "10762" },
                    _ => gid,
                };

                if safe_kind == "tv" {
                    match resolved_genre_id {
                        // Docuseries: require vote threshold and modern era to avoid 1-vote local foreign broadcasts
                        "99" => {
                            let vote_filter = if has_custom_votes { "" } else { "&vote_count.gte=25" };
                            format!("{}/discover/tv?api_key={}&with_genres=99{}&first_air_date.gte=2000-01-01&include_adult=false&sort_by={}&page={}", TMDB_BASE_URL, TMDB_API_KEY, vote_filter, default_sort, page)
                        }
                        // Action & Adventure TV: exclude animation & kids cartoons (Doraemon, etc.)
                        "10759" => {
                            let vote_filter = if has_custom_votes { "" } else { "&vote_count.gte=35" };
                            format!("{}/discover/tv?api_key={}&with_genres=10759&without_genres=16,10762{}&include_adult=false&sort_by={}&page={}", TMDB_BASE_URL, TMDB_API_KEY, vote_filter, default_sort, page)
                        }
                        // Kids & Family TV: modern era threshold to avoid 1960s relics
                        "10762" => {
                            let vote_filter = if has_custom_votes { "" } else { "&vote_count.gte=20" };
                            format!("{}/discover/tv?api_key={}&with_genres=10762&first_air_date.gte=1990-01-01{}&include_adult=false&sort_by={}&page={}", TMDB_BASE_URL, TMDB_API_KEY, vote_filter, default_sort, page)
                        }
                        // Sci-Fi & Fantasy TV: exclude pure animation to keep live-action series prominent
                        "10765" => {
                            let vote_filter = if has_custom_votes { "" } else { "&vote_count.gte=30" };
                            format!("{}/discover/tv?api_key={}&with_genres=10765&without_genres=16{}&include_adult=false&sort_by={}&page={}", TMDB_BASE_URL, TMDB_API_KEY, vote_filter, default_sort, page)
                        }
                        // Comedies & Dramas: exclude reality/talk shows
                        "35" => {
                            let vote_filter = if has_custom_votes { "" } else { "&vote_count.gte=30" };
                            format!("{}/discover/tv?api_key={}&with_genres=35&without_genres=10764,10767{}&include_adult=false&sort_by={}&page={}", TMDB_BASE_URL, TMDB_API_KEY, vote_filter, default_sort, page)
                        }
                        "18" => {
                            let vote_filter = if has_custom_votes { "" } else { "&vote_count.gte=30" };
                            format!("{}/discover/tv?api_key={}&with_genres=18&without_genres=10764,10767{}&include_adult=false&sort_by={}&page={}", TMDB_BASE_URL, TMDB_API_KEY, vote_filter, default_sort, page)
                        }
                        _ => {
                            let vote_filter = if has_custom_votes { "" } else { "&vote_count.gte=25" };
                            format!("{}/discover/tv?api_key={}&with_genres={}{}&include_adult=false&sort_by={}&page={}", TMDB_BASE_URL, TMDB_API_KEY, resolved_genre_id, vote_filter, default_sort, page)
                        }
                    }
                } else {
                    let vote_filter = if has_custom_votes { "" } else { "&vote_count.gte=35" };
                    format!("{}/discover/movie?api_key={}&with_genres={}{}&include_adult=false&sort_by={}&page={}", TMDB_BASE_URL, TMDB_API_KEY, resolved_genre_id, vote_filter, default_sort, page)
                }
            }
        };

        if let Some(extra) = extra_params {
            if !extra.is_empty() {
                let extra_clean = extra.trim_start_matches('&');
                url.push_str(&format!("&{}", extra_clean));
            }
        }

        if let Ok(resp) = get(&url).send().await {
            if let Ok(data) = resp.json::<TmdbResponse<MediaItem>>().await {
                let data = data.rated().await;
                let items: Vec<MediaItem> = data.results.into_iter().filter_map(|mut item| {
                    if item.is_nsfw() {
                        return None;
                    }
                    if item.backdrop_path.is_none() && item.poster_path.is_none() {
                        return None;
                    }
                    if item.media_type.is_none() {
                        item.media_type = Some(safe_kind.to_string());
                    }
                    Some(item)
                }).collect();
                if !items.is_empty() {
                    return Ok(items);
                }
            }
        }

        // If extra_params caused 0 results, retry without the restrictive extra_params
        if extra_params.is_some() {
            // Strip extra_params from url and retry
            let base_url = if url.contains('&') {
                if let Some(extra) = extra_params {
                    url.replace(extra, "")
                } else {
                    url.clone()
                }
            } else {
                url.clone()
            };
            if let Ok(resp) = get(&base_url).send().await {
                if let Ok(data) = resp.json::<TmdbResponse<MediaItem>>().await {
                let data = data.rated().await;
                    let items: Vec<MediaItem> = data.results.into_iter().filter_map(|mut item| {
                        if item.is_nsfw() || (item.backdrop_path.is_none() && item.poster_path.is_none()) {
                            None
                        } else {
                            if item.media_type.is_none() {
                                item.media_type = Some(safe_kind.to_string());
                            }
                            Some(item)
                        }
                    }).collect();
                    if !items.is_empty() {
                        return Ok(items);
                    }
                }
            }
        }
    }

    // 3. Fallback: ALWAYS respect safe_kind so TV series never show movies
    fetch_trending_page(safe_kind, page).await
}

pub async fn fetch_for_genre_route_page(genre_key: &str, page: u32) -> Result<Vec<MediaItem>, gloo_net::Error> {
    fetch_row_content("movie", Some(genre_key), None, None, None, page).await
}

pub async fn fetch_by_language(lang: &str, page: u32) -> Result<Vec<MediaItem>, gloo_net::Error> {
    let url_movie = format!(
        "{}/discover/movie?api_key={}&with_original_language={}&sort_by=popularity.desc&page={}",
        TMDB_BASE_URL, TMDB_API_KEY, lang, page
    );
    let resp_movie: Result<TmdbResponse<MediaItem>, _> = get(&url_movie).send().await?.json().await;

    let url_tv = format!(
        "{}/discover/tv?api_key={}&with_original_language={}&sort_by=popularity.desc&page={}",
        TMDB_BASE_URL, TMDB_API_KEY, lang, page
    );
    let resp_tv: Result<TmdbResponse<MediaItem>, _> = get(&url_tv).send().await?.json().await;

    let resp_movie = match resp_movie { Ok(r) => Ok(r.rated().await), e => e };
    let resp_tv = match resp_tv { Ok(r) => Ok(r.rated().await), e => e };
    let mut combined = resp_movie.map(|r| r.results).unwrap_or_default();
    if let Ok(tv_resp) = resp_tv {
        for mut item in tv_resp.results {
            item.media_type = Some("tv".to_string());
            combined.push(item);
        }
    }
    Ok(combined)
}


impl TmdbResponse<MediaItem> {
    /// Drops the titles the profile in use may not see (its age rating limit), see `services::rating`.
    pub async fn rated(mut self) -> Self {
        self.results = crate::services::rating::filter_allowed(std::mem::take(&mut self.results)).await;
        self
    }
}
