use gloo_net::http::Request;
use serde::{Deserialize, Serialize};

const RESOLVER_BASE_URL: &str = crate::config::RESOLVER_URL;

/// Where the resolver lives, for the help center health check.
pub fn resolver_base_url() -> &'static str {
    RESOLVER_BASE_URL
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RawStreamResponse {
    pub success: Option<bool>,
    pub provider: Option<String>,
    pub sources: Option<Vec<RawSource>>,
    pub subtitles: Option<Vec<RawSubtitle>>,
    pub error: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RawSource {
    pub url: Option<String>,
    pub quality: Option<String>,
    #[serde(rename = "isM3U8")]
    pub is_m3u8: Option<bool>,
    #[serde(rename = "isEmbed")]
    pub is_embed: Option<bool>,
    #[serde(rename = "noProxy")]
    pub no_proxy: Option<bool>,
    pub provider: Option<String>,
    #[serde(rename = "providerId")]
    pub provider_id: Option<String>,
    pub referer: Option<String>,
    #[serde(default)]
    pub audio: Option<String>,
    #[serde(default, rename = "audioLanguages")]
    pub audio_languages: Vec<String>,
    #[serde(default, rename = "isOriginal")]
    pub is_original: Option<bool>,
    #[serde(default, rename = "isMultiAudio")]
    pub is_multi_audio: Option<bool>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RawSubtitle {
    pub url: Option<String>,
    pub lang: Option<String>,
    pub label: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlayableKind {
    Hls,
    Mp4,
    Embed,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PlayableSource {
    pub url: String,
    pub quality: String,
    pub provider: String,
    pub kind: PlayableKind,
    pub referer: Option<String>,
    pub audio: Option<String>,
    pub audio_languages: Vec<String>,
    pub is_original: Option<bool>,
    pub is_multi_audio: Option<bool>,
}

impl PlayableSource {
    pub fn rank_score(&self) -> i32 {
        let quality_score = match self.quality.to_lowercase().as_str() {
            "4k" | "2160p" => 50,
            "1080p" => 40,
            "auto" => 35,
            "720p" => 30,
            "480p" => 20,
            "360p" => 10,
            _ => 25,
        };

        let audio_bonus = if self.is_multi_audio.unwrap_or(false)
            || self.audio.as_deref() == Some("multi")
            || self.is_original == Some(true)
            || self.audio.as_deref() == Some("en")
        {
            60
        } else if self.audio.is_none() {
            20
        } else {
            -40 // foreign-only dub
        };

        let kind_score = match self.kind {
            PlayableKind::Hls => 100,
            PlayableKind::Mp4 => 80,
            PlayableKind::Embed => 10,
        };

        kind_score + quality_score + audio_bonus
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SubtitleTrack {
    pub url: String,
    pub lang: String,
    pub label: String,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct StreamResolveResult {
    pub sources: Vec<PlayableSource>,
    pub subtitles: Vec<SubtitleTrack>,
    pub provider: Option<String>,
}

impl StreamResolveResult {
    pub fn primary_source(&self) -> Option<&PlayableSource> {
        self.sources.first()
    }
}

pub async fn resolve_stream(
    tmdb_id: u32,
    is_tv: bool,
    title: &str,
    year: Option<&str>,
    season: Option<u32>,
    episode: Option<u32>,
    force: bool,
    orig_lang: Option<&str>,
) -> Result<StreamResolveResult, String> {
    if tmdb_id == 0 {
        return Err("Invalid media ID".to_string());
    }

    let media_type = if is_tv { "tv" } else { "movie" };
    let mut url = format!(
        "{}/api/stream?tmdbId={}&type={}&title={}",
        RESOLVER_BASE_URL,
        tmdb_id,
        media_type,
        js_sys::encode_uri_component(title),
    );

    if let Some(y) = year {
        if !y.is_empty() {
            url.push_str(&format!("&year={}", js_sys::encode_uri_component(y)));
        }
    }

    if let Some(ol) = orig_lang {
        if !ol.is_empty() {
            url.push_str(&format!("&orig_lang={}", js_sys::encode_uri_component(ol)));
        }
    }

    if is_tv {
        let s = season.unwrap_or(1);
        let e = episode.unwrap_or(1);
        url.push_str(&format!("&season={}&episode={}", s, e));
    }

    if force {
        url.push_str("&force=1");
    }

    let resp = Request::get(&url)
        .send()
        .await
        .map_err(|e| format!("Failed to reach stream resolver: {}", e))?;

    if !resp.ok() {
        return Err(format!("Stream resolver returned HTTP status {}", resp.status()));
    }

    let data: RawStreamResponse = resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse stream response: {}", e))?;

    if data.success == Some(false) && data.sources.as_ref().map_or(true, |s| s.is_empty()) {
        return Err(data.error.unwrap_or_else(|| "No playable stream found for this title.".to_string()));
    }

    let mut playable_sources: Vec<PlayableSource> = Vec::new();

    if let Some(raw_sources) = data.sources {
        for s in raw_sources {
            if let Some(u) = s.url {
                if u.trim().is_empty() {
                    continue;
                }

                // Exclude MPEG-DASH (.mpd) directly as browsers do not natively play dash without dash.js
                let is_mpd = u.contains(".mpd");
                if is_mpd {
                    continue;
                }

                let is_embed = s.is_embed.unwrap_or(false) || u.contains("/embed/");
                let is_m3u8 = s.is_m3u8.unwrap_or(false) || u.contains(".m3u8");

                let kind = if is_embed {
                    PlayableKind::Embed
                } else if is_m3u8 {
                    PlayableKind::Hls
                } else {
                    PlayableKind::Mp4
                };

                let quality = s.quality.unwrap_or_else(|| "auto".to_string());
                let provider = s.provider.unwrap_or_else(|| "Stream Provider".to_string());

                playable_sources.push(PlayableSource {
                    url: u,
                    quality,
                    provider,
                    kind,
                    referer: s.referer,
                    audio: s.audio,
                    audio_languages: s.audio_languages,
                    is_original: s.is_original,
                    is_multi_audio: s.is_multi_audio,
                });
            }
        }
    }

    // Sort sources by rank score descending (highest quality direct HLS first, down to embed)
    playable_sources.sort_by(|a, b| b.rank_score().cmp(&a.rank_score()));

    let mut subtitle_tracks: Vec<SubtitleTrack> = Vec::new();
    if let Some(raw_subs) = data.subtitles {
        for sub in raw_subs {
            if let (Some(u), Some(lang)) = (sub.url, sub.lang) {
                if !u.trim().is_empty() {
                    let label = sub.label.unwrap_or_else(|| lang.clone());
                    subtitle_tracks.push(SubtitleTrack {
                        url: u,
                        lang,
                        label,
                    });
                }
            }
        }
    }

    if playable_sources.is_empty() {
        return Err(data.error.unwrap_or_else(|| "All stream providers are currently unavailable for this title.".to_string()));
    }

    Ok(StreamResolveResult {
        sources: playable_sources,
        subtitles: subtitle_tracks,
        provider: data.provider,
    })
}
