//! A title's videos and the choice of its trailer (refused trailers are skipped, services/unplayable.rs).
use super::*;

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
        // a video YouTube refused here before is not offered again (services/unplayable.rs)
        .filter(|v| !crate::services::unplayable::is_unplayable(&v.key))
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

/// Names of videos filed as clips that are no scene: the film's opening, a whole film preview, ticket and countdown promos.
const NOT_A_SCENE: [&str; 10] = ["minutes", "full movie", "tickets", "in theaters", "in theatres", "in cinemas", "now playing", "tv spot", " days", "countdown"];

/// The video for a title's place in Clips: a scene from it, as Netflix's Clips are, when TMDB lists one ("Clip"; 14 of the 20 titles in
/// the feed on 2026-10-24, note 116), else its trailer. Scenes named as clips or previews come first; openings ("First 10 Minutes") and
/// promos ("IN 10 DAYS!") are passed over. Returns (video_key, is_scene).
pub fn select_clip(videos: &[VideoResult]) -> Option<(String, bool)> {
    let scene = videos
        .iter()
        .filter(|v| v.site.eq_ignore_ascii_case("youtube") && !v.key.trim().is_empty() && v.r#type.eq_ignore_ascii_case("clip"))
        .filter(|v| !crate::services::unplayable::is_unplayable(&v.key))
        .filter(|v| {
            let name = v.name.as_deref().unwrap_or("").to_lowercase();
            !NOT_A_SCENE.iter().any(|w| name.contains(w))
        })
        .max_by_key(|v| {
            let name = v.name.as_deref().unwrap_or("").to_lowercase();
            let mut score = 0i32;
            if v.official == Some(true) {
                score += 50;
            }
            if name.contains("clip") || name.contains("scene") || name.contains("preview") {
                score += 30;
            }
            score
        });
    match scene {
        Some(v) => Some((v.key.clone(), true)),
        None => select_best_tmdb_trailer(videos).map(|(k, _)| (k, false)),
    }
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
    let resp: TmdbVideos = get_json(&url).await?;
    Ok(resp.results)
}
