//! Subtitles in the player: the list (the stream's own tracks, then SubDL through the resolver, one entry per language), the file
//! (interop/subtitles.ts downloads it, decodes it and checks it holds real cues, trying a language's next file when one does not),
//! and the line on screen, drawn in the profile's Subtitle appearance and moved up while the controls show.
//!
//! The player's own `subtitles` list of `SubtitleTrack`s stays the one the drawers show: a SubDL language is a track whose url is
//! `subdl:<CODE>`, and its files are kept in `Candidates`.

use crate::services::resolver::SubtitleTrack;
use leptos::prelude::*;
use std::collections::HashMap;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(module = "/interop/dist/streams-interop.js")]
extern "C" {
    #[wasm_bindgen(js_name = subtitleLoad)]
    async fn subtitle_load(candidates: &str) -> JsValue;
}

/// The key of a SubDL language in the track list: `subdl:EN`.
pub const SUBDL: &str = "subdl:";

/// The files of each track key, in the order they are tried: (address, language code).
pub type Candidates = HashMap<String, Vec<(String, String)>>;

#[derive(Clone, Debug, PartialEq, serde::Deserialize)]
pub struct Cue {
    pub s: f64,
    pub e: f64,
    pub t: String,
}

#[derive(serde::Deserialize)]
struct Loaded {
    cues: Vec<Cue>,
}

#[derive(serde::Deserialize)]
struct SubdlFile {
    language: String,
    url: String,
    #[serde(default)]
    zip: bool,
}

#[derive(serde::Deserialize)]
struct SubdlAnswer {
    #[serde(default)]
    subtitles: Vec<SubdlFile>,
}

/// The languages asked of SubDL: the profile's subtitle language first, then the languages the site is written in (SubDL answers at
/// most 30 files, three per language).
fn wanted_languages(preferred: Option<&str>) -> String {
    let mut codes: Vec<String> = preferred.filter(|p| !p.is_empty() && *p != "off").map(|p| p.to_ascii_uppercase()).into_iter().collect();
    for c in ["EN", "AR", "DE", "ES", "FR", "HI", "IT", "JA", "KO", "NL", "PL", "PT", "RU", "SV", "TR", "ZH"] {
        if !codes.iter().any(|x| x == c) {
            codes.push(c.to_string());
        }
    }
    codes.join(",")
}

/// SubDL's files for this title or episode, as tracks (one per language) and their candidates. Empty when the resolver has nothing.
pub async fn subdl_tracks(media_id: u32, is_tv: bool, season: Option<u32>, episode: Option<u32>, preferred: Option<String>) -> (Vec<SubtitleTrack>, Candidates) {
    let mut url = format!(
        "{}/api/subtitles/subdl?tmdbId={media_id}&type={}&langs={}",
        crate::config::RESOLVER_URL,
        if is_tv { "tv" } else { "movie" },
        wanted_languages(preferred.as_deref())
    );
    if is_tv {
        url.push_str(&format!("&season={}&episode={}", season.unwrap_or(1), episode.unwrap_or(1)));
    }
    let token = crate::services::account::ensure_fresh().await.map(|s| s.access_token);
    let mut request = gloo_net::http::Request::get(&url);
    if let Some(t) = token.as_deref().filter(|t| !t.is_empty()) {
        request = request.header("Authorization", &format!("Bearer {t}"));
    }
    let Ok(resp) = request.send().await else { return (Vec::new(), HashMap::new()) };
    if !resp.ok() {
        return (Vec::new(), HashMap::new());
    }
    let Ok(answer) = resp.json::<SubdlAnswer>().await else { return (Vec::new(), HashMap::new()) };
    let mut order: Vec<String> = Vec::new();
    let mut candidates: Candidates = HashMap::new();
    for f in answer.subtitles.into_iter().filter(|f| !f.zip && f.url.starts_with("https://dl.subdl.com/")) {
        let code = f.language.to_ascii_uppercase();
        let key = format!("{SUBDL}{code}");
        if !candidates.contains_key(&key) {
            order.push(code.clone());
        }
        candidates.entry(key).or_default().push((f.url, code));
    }
    let tracks = order
        .into_iter()
        .map(|code| SubtitleTrack { url: format!("{SUBDL}{code}"), label: crate::i18n::language_name(&code.to_ascii_lowercase()), lang: code.to_ascii_lowercase() })
        .collect();
    (tracks, candidates)
}

/// The cue showing at `t` seconds, if any (cues are sorted by start; a few overlap, the latest started wins).
pub fn cue_at(cues: &[Cue], t: f64) -> Option<usize> {
    let after = cues.partition_point(|c| c.s <= t);
    (0..after).rev().take(4).find(|&i| cues[i].e > t)
}

/// A cue's text as HTML: everything escaped, then italic, bold and underline put back, lines broken.
pub fn cue_html(text: &str) -> String {
    let mut out = text.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;");
    for tag in ["i", "b", "u", "/i", "/b", "/u"] {
        out = out.replace(&format!("&lt;{tag}&gt;"), &format!("<{tag}>")).replace(&format!("&lt;{}&gt;", tag.to_uppercase()), &format!("<{tag}>"));
    }
    out.replace('\n', "<br>")
}

/// The line on screen. `selected` is a track key (a provider address or `subdl:<CODE>`), `delay` moves every cue (seconds, positive
/// shows them later), `lifted` moves the line above the controls while they show.
#[component]
pub fn SubtitleOverlay(selected: ReadSignal<Option<String>>, candidates: StoredValue<Candidates>, delay: ReadSignal<f64>, lifted: Signal<bool>) -> impl IntoView {
    let prefs = crate::pages::settings::use_prefs();
    let profiles = crate::store::use_profile_store();
    let style = Memo::new(move |_| {
        let id = profiles.active_profile_id.get().unwrap_or_default();
        crate::pages::settings::subtitles::player_cue_css(&prefs.profile(&id).subtitle_style)
    });
    let cues = RwSignal::new(None::<std::sync::Arc<Vec<Cue>>>);
    let (line, set_line) = signal(None::<usize>);

    // the file of the selected track: loaded when the choice changes, the previous one dropped at once
    Effect::new(move |_| {
        let key = selected.get();
        cues.set(None);
        set_line.set(None);
        let Some(key) = key else { return };
        let list: Vec<(String, String)> = if key.starts_with(SUBDL) {
            candidates.with_value(|c| c.get(&key).cloned().unwrap_or_default())
        } else {
            vec![(key.clone(), String::new())]
        };
        let json = serde_json::to_string(&list.iter().map(|(url, lang)| serde_json::json!({ "url": url, "lang": lang })).collect::<Vec<_>>()).unwrap_or_default();
        leptos::task::spawn_local(async move {
            let loaded = subtitle_load(&json).await.as_string().unwrap_or_default();
            // the choice may have changed while the file loaded
            if selected.get_untracked().as_deref() != Some(key.as_str()) {
                return;
            }
            if let Ok(l) = serde_json::from_str::<Loaded>(&loaded) {
                cues.set(Some(std::sync::Arc::new(l.cues)));
            }
        });
    });

    // ten times a second the video's own clock picks the line (the player's time signal moves four times a second)
    let tick = leptos::prelude::set_interval_with_handle(
        move || {
            let Some(list) = cues.get_untracked() else { return };
            let Some(video) = web_sys::window()
                .and_then(|w| w.document())
                .and_then(|d| d.get_element_by_id(crate::keys::VIDEO_ELEMENT_ID))
                .and_then(|e| e.dyn_into::<web_sys::HtmlVideoElement>().ok())
            else {
                return;
            };
            let now = cue_at(&list, video.current_time() - delay.get_untracked());
            if now != line.get_untracked() {
                set_line.set(now);
            }
        },
        std::time::Duration::from_millis(100),
    );
    on_cleanup(move || {
        if let Ok(t) = tick {
            t.clear();
        }
    });

    view! {
        <div class="pointer-events-none absolute inset-0 z-10" style="container-type:size" aria-live="off">
            <div
                class="absolute inset-x-0 flex justify-center px-[6%] text-center transition-[bottom] duration-(--motion-base)"
                style=move || if lifted.get() { "bottom:20%" } else { "bottom:7%" }
            >
                {move || {
                    let text = line.get().and_then(|i| cues.with(|c| c.as_ref().and_then(|c| c.get(i).map(|cue| cue.t.clone()))));
                    text.map(|t| view! { <span class="inline-block font-semibold [unicode-bidi:plaintext]" style=move || style.get() inner_html=cue_html(&t) /> })
                }}
            </div>
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cue(s: f64, e: f64) -> Cue {
        Cue { s, e, t: String::new() }
    }

    #[test]
    fn finds_the_line_showing() {
        let cues = vec![cue(1.0, 2.0), cue(3.0, 5.0), cue(4.0, 4.5)];
        assert_eq!(cue_at(&cues, 0.5), None);
        assert_eq!(cue_at(&cues, 1.5), Some(0));
        assert_eq!(cue_at(&cues, 2.5), None);
        assert_eq!(cue_at(&cues, 4.2), Some(2));
        assert_eq!(cue_at(&cues, 4.8), Some(1));
    }

    #[test]
    fn keeps_only_simple_styling() {
        assert_eq!(cue_html("<i>Hi</i> & <script>x</script>\nthere"), "<i>Hi</i> &amp; &lt;script&gt;x&lt;/script&gt;<br>there");
    }

    #[test]
    fn asks_the_profiles_language_first() {
        assert!(wanted_languages(Some("fr")).starts_with("FR,EN,AR"));
        assert!(wanted_languages(Some("off")).starts_with("EN,"));
    }
}
