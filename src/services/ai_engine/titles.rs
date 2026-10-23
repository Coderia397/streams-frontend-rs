//! Single lists of titles: search, a language's titles, games, and a title's skip marks and content reasons.
use super::*;

/// A game of the Games tab (from the engine's `games.json`).
#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct Game {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub image: String,
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub genre: String,
}

/// Where the skippable parts of a video sit, as ratios of its length (`intro` and `recap` as [start, end], `credits` as the start), from the engine's guesses (note 71); all empty for an unknown title.
#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
pub struct SkipMarks {
    pub intro: Option<[f64; 2]>,
    pub recap: Option<[f64; 2]>,
    pub credits: Option<f64>,
}

/// The skip marks of one video; `last` says the episode is the last of its season. Empty when the engine has none or cannot be reached.
pub async fn fetch_skip_marks(is_tv: bool, id: u32, episode: Option<u32>, last: bool) -> SkipMarks {
    let mut url = format!("{}/api/skip-marks?type={}&id={}", base(), if is_tv { "tv" } else { "movie" }, id);
    if let Some(e) = episode { url.push_str(&format!("&episode={e}")); }
    if last { url.push_str("&last=1"); }
    engine_json::<SkipMarks>(&url, MENU_FRESH_MS).await.unwrap_or_default()
}

/// The content reasons of one video (words of the fixed list of 20, in list order), from the engine's guesses (note 69); empty when there is no answer, so only the rating shows.
pub async fn fetch_content_reasons(is_tv: bool, id: u32, season: Option<u32>, episode: Option<u32>) -> Vec<String> {
    #[derive(Deserialize)]
    struct Answer { reasons: Option<Vec<String>> }
    let mut url = format!("{}/api/content-reasons?type={}&id={}", base(), if is_tv { "tv" } else { "movie" }, id);
    if let Some(s) = season { url.push_str(&format!("&season={s}")); }
    if let Some(e) = episode { url.push_str(&format!("&episode={e}")); }
    engine_json::<Answer>(&url, MENU_FRESH_MS).await.and_then(|a| a.reasons).unwrap_or_default()
}

/// The games the engine offers; empty while Games is off.
pub async fn fetch_games() -> Vec<Game> {
    #[derive(Deserialize)]
    struct Answer { games: Vec<Game> }
    engine_json::<Answer>(&format!("{}/api/games", base()), MENU_FRESH_MS).await.map(|a| a.games).unwrap_or_default()
}

/// Search from the engine: titles by words (the last one may be unfinished), by director or cast, and near misses for typos, with the real
/// certificates and the profile's age limit applied. None when the engine cannot be reached.
pub async fn search_titles(query: &str) -> Option<Vec<MediaItem>> {
    let encoded: String = js_sys::encode_uri_component(query.trim()).into();
    let url = format!("{}/api/search?q={}&limit=40{}", base(), encoded, rating_query());
    fetch_result_list(&url).await
}

/// What the search page offers before anything is typed: titles the engine picks for this session (taste, not started, well known), different every six hours.
pub async fn fetch_search_suggestions(limit: usize) -> Option<Vec<MediaItem>> {
    let url = format!("{}/api/search-suggestions?session_id={}&limit={}{}", base(), get_or_create_session_id(), limit, rating_query());
    fetch_result_list(&url).await
}

/// A `{ "results": [...] }` answer of the engine (search, suggestions) as media items; titles with no picture are left out.
async fn fetch_result_list(url: &str) -> Option<Vec<MediaItem>> {
    let v: serde_json::Value = engine_json(url, FEED_FRESH_MS).await?;
    let text = |r: &serde_json::Value, k: &str| r.get(k).and_then(|x| x.as_str()).map(str::to_string);
    let items = v
        .get("results")?
        .as_array()?
        .iter()
        .filter_map(|r| {
            let media_type = text(r, "media_type")?;
            let year = r.get("release_year").and_then(|y| y.as_i64()).map(|y| y.to_string());
            let title = text(r, "title");
            Some(MediaItem {
                id: r.get("id")?.as_u64()? as u32,
                title: if media_type == "tv" { None } else { title.clone() },
                name: if media_type == "tv" { title } else { None },
                overview: text(r, "overview").unwrap_or_default(),
                poster_path: text(r, "poster_path").filter(|p| !p.is_empty()),
                backdrop_path: text(r, "backdrop_path").filter(|p| !p.is_empty()),
                vote_average: r.get("vote_average").and_then(|x| x.as_f64()).unwrap_or(0.0),
                release_date: year.clone(),
                first_air_date: year,
                media_type: Some(media_type),
                rating_gb: text(r, "rating_gb"),
                rating_us: text(r, "rating_us"),
                ..Default::default()
            })
        })
        .filter(|m| m.poster_path.is_some() || m.backdrop_path.is_some())
        .collect();
    Some(items)
}

/// The titles of a language page (original language) as rows from the engine, the Top 10 rows left out so the page reads as one ranked grid.
pub async fn fetch_language_titles(lang: &str) -> Option<Vec<MediaItem>> {
    let url = format!("{}/api/feed?language={}&limit=12&session_id={}{}", base(), lang, get_or_create_session_id(), rating_query());
    let data: FeedResponse = engine_json(&url, FEED_FRESH_MS).await?;
    let mut seen = std::collections::HashSet::new();
    let items: Vec<MediaItem> = data
        .rows
        .iter()
        .filter(|r| !r.is_top_ten())
        .flat_map(|r| r.to_media_items())
        .filter(|m| seen.insert((m.media_type.clone(), m.id)))
        .collect();
    (!items.is_empty()).then_some(items)
}
