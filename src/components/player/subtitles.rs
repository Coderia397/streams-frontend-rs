//! Subtitles in the player: one list with one entry per language, the file, and the line on screen.
//!
//! The list joins the stream's own tracks and SubDL's files (through the resolver, which keeps the key). Every language label goes
//! through `utils::lang_tag`, so "BR_PT", "pob" and "Portuguese (Brazil)" are one entry, named in the viewer's language. The file:
//! interop/subtitles.ts downloads it, decodes it, and keeps it only when it holds real cues, in the language it is listed as (a file
//! listed as English held Persian on 2026-10-25), that fit the film; otherwise the language's next file is tried. The line is drawn
//! in the profile's Subtitle appearance and moves up while the controls show.
//!
//! A language is a `SubtitleTrack` whose url is `sub:<tag>` (`sub:en`, `sub:pt-BR`); its files, in the order they are tried, are in
//! `Candidates`.

use crate::services::resolver::SubtitleTrack;
use crate::utils::lang_tag::{self, Lang};
use leptos::prelude::*;
use std::collections::HashMap;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(module = "/interop/dist/streams-interop.js")]
extern "C" {
    #[wasm_bindgen(js_name = subtitleLoad)]
    async fn subtitle_load(candidates: &str, duration: f64) -> JsValue;
    #[wasm_bindgen(js_name = subtitleNativeTrack)]
    fn subtitle_native_track(video_id: &str, cues: &str, delay: f64);
}

/// The start of a language's key in the track list: `sub:en`.
pub const KEY: &str = "sub:";

/// The files of each track key, in the order they are tried: (address, language tag).
pub type Candidates = HashMap<String, Vec<(String, String)>>;

/// One subtitle file and the language it is listed as.
#[derive(Clone, Debug, PartialEq)]
pub struct SubFile {
    pub url: String,
    pub lang: Lang,
}

#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Cue {
    pub s: f64,
    pub e: f64,
    pub t: String,
}

#[derive(serde::Deserialize)]
struct Loaded {
    #[serde(default)]
    index: usize,
    cues: Vec<Cue>,
}

#[derive(serde::Deserialize)]
struct SubdlFile {
    language: String,
    #[serde(default)]
    label: String,
    url: String,
    #[serde(default)]
    zip: bool,
}

#[derive(serde::Deserialize)]
struct SubdlAnswer {
    #[serde(default)]
    subtitles: Vec<SubdlFile>,
}

/// The languages asked of SubDL (its codes): the ones the viewer most likely wants first, then the languages the site is written in.
/// SubDL answers at most 30 files, three per language.
fn wanted_languages(first: &[String]) -> String {
    let mut codes: Vec<String> = Vec::new();
    let site = ["en", "ar", "de", "es", "fr", "hi", "it", "ja", "ko", "nl", "pl", "pt", "ru", "sv", "tr", "zh"];
    for tag in first.iter().map(String::as_str).chain(site) {
        let code = lang_tag::subdl_code(tag);
        if !codes.contains(&code) {
            codes.push(code);
        }
    }
    codes.join(",")
}

/// SubDL's files for this title or episode. Empty when the resolver has nothing or cannot be reached.
pub async fn subdl_files(media_id: u32, is_tv: bool, season: Option<u32>, episode: Option<u32>, first: &[String]) -> Vec<SubFile> {
    let mut url = format!(
        "{}/api/subtitles/subdl?tmdbId={media_id}&type={}&langs={}",
        crate::config::RESOLVER_URL,
        if is_tv { "tv" } else { "movie" },
        wanted_languages(first)
    );
    if is_tv {
        url.push_str(&format!("&season={}&episode={}", season.unwrap_or(1), episode.unwrap_or(1)));
    }
    let token = crate::services::account::ensure_fresh().await.map(|s| s.access_token);
    let mut request = gloo_net::http::Request::get(&url);
    if let Some(t) = token.as_deref().filter(|t| !t.is_empty()) {
        request = request.header("Authorization", &format!("Bearer {t}"));
    }
    let Ok(resp) = request.send().await else { return Vec::new() };
    if !resp.ok() {
        return Vec::new();
    }
    let Ok(answer) = resp.json::<SubdlAnswer>().await else { return Vec::new() };
    answer
        .subtitles
        .into_iter()
        // single files only: a zip would need unpacking in the browser
        .filter(|f| {
            !f.zip
                && (f.url.starts_with("https://dl.subdl.com/")
                    || f.url.contains("/api/subtitles/download")
                    || f.url.starts_with(crate::config::RESOLVER_URL))
        })
        .filter_map(|f| {
            let lang = lang_tag::parse(&f.language).or_else(|| lang_tag::parse(&f.label))?;
            Some(SubFile { url: f.url, lang })
        })
        .collect()
}

/// The stream's own subtitle tracks as files (their language read from the code or, failing that, the label).
pub fn stream_files(tracks: &[SubtitleTrack]) -> Vec<SubFile> {
    tracks
        .iter()
        .filter(|t| t.url.starts_with("http"))
        .filter_map(|t| Some(SubFile { url: t.url.clone(), lang: lang_tag::parse(&t.lang).or_else(|| lang_tag::parse(&t.label))? }))
        .collect()
}

/// The list the drawers show (one entry per language) and the files behind each. Forced files are left out (they hold only the lines
/// spoken in another language than the audio); files come in the order given (the stream's own first), those for hearing impaired
/// viewers after the others. The languages of `first` lead in that order; the rest follow by name.
pub fn build(files: &[SubFile], first: &[String], name: impl Fn(&str) -> String) -> (Vec<SubtitleTrack>, Candidates) {
    let mut by_tag: Vec<(String, Vec<&SubFile>)> = Vec::new();
    for f in files.iter().filter(|f| !f.lang.forced) {
        match by_tag.iter_mut().find(|(t, _)| *t == f.lang.tag) {
            Some((_, list)) => {
                if !list.iter().any(|x| x.url == f.url) {
                    list.push(f);
                }
            }
            None => by_tag.push((f.lang.tag.clone(), vec![f])),
        }
    }
    let rank = |tag: &str| {
        first.iter().position(|w| w == tag).map(|i| (0, i)).or_else(|| first.iter().position(|w| lang_tag::base(w) == lang_tag::base(tag)).map(|i| (1, i))).unwrap_or((2, 0))
    };
    let mut entries: Vec<(String, String, Vec<&SubFile>)> = by_tag.into_iter().map(|(tag, mut list)| {
        list.sort_by_key(|f| f.lang.hi);
        (name(&tag), tag, list)
    }).collect();
    entries.sort_by(|a, b| rank(&a.1).cmp(&rank(&b.1)).then_with(|| a.0.to_lowercase().cmp(&b.0.to_lowercase())));
    let mut candidates = Candidates::new();
    let tracks = entries
        .into_iter()
        .map(|(label, tag, list)| {
            let key = format!("{KEY}{tag}");
            candidates.insert(key.clone(), list.iter().map(|f| (f.url.clone(), tag.clone())).collect());
            SubtitleTrack { url: key, label, lang: tag }
        })
        .collect();
    (tracks, candidates)
}

/// The key of the language to show for `wanted` (a tag, or a language whose version is not there: `pt` finds `pt-BR`).
pub fn key_for(candidates: &Candidates, wanted: &str) -> Option<String> {
    let exact = format!("{KEY}{wanted}");
    if candidates.contains_key(&exact) {
        return Some(exact);
    }
    let base = lang_tag::base(wanted);
    let mut keys: Vec<&String> = candidates.keys().filter(|k| k.strip_prefix(KEY).is_some_and(|t| lang_tag::base(t) == base)).collect();
    keys.sort();
    keys.first().map(|k| k.to_string())
}

fn video_element() -> Option<web_sys::HtmlVideoElement> {
    web_sys::window()?.document()?.get_element_by_id(crate::keys::VIDEO_ELEMENT_ID)?.dyn_into::<web_sys::HtmlVideoElement>().ok()
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
pub fn SubtitleOverlay(
    selected: ReadSignal<Option<String>>,
    candidates: Signal<Candidates>,
    delay: ReadSignal<f64>,
    lifted: Signal<bool>,
    /// bumped to move to the chosen language's next file (Out of sync? Try another version)
    retry: ReadSignal<u32>,
) -> impl IntoView {
    let prefs = crate::pages::settings::use_prefs();
    let profiles = crate::store::use_profile_store();
    let style = Memo::new(move |_| {
        let id = profiles.active_profile_id.get().unwrap_or_default();
        crate::pages::settings::subtitles::player_cue_css(&prefs.profile(&id).subtitle_style)
    });
    let cues = RwSignal::new(None::<std::sync::Arc<Vec<Cue>>>);
    // the file showing: the language's key and its place in that language's list
    let showing = StoredValue::new(None::<(String, usize)>);
    let (line, set_line) = signal(None::<usize>);

    // the file of the selected track: loaded when the choice changes, the previous one dropped at once
    Effect::new(move |_| {
        let key = selected.get();
        let again = retry.get();
        cues.set(None);
        set_line.set(None);
        let Some(key) = key else { return };
        let all: Vec<(String, String)> = candidates.with_untracked(|c| c.get(&key).cloned().unwrap_or_default());
        if all.is_empty() {
            return;
        }
        // a retry starts after the file showing; a new choice starts at the language's first file
        let start = match showing.get_value() {
            Some((k, i)) if k == key && again > 0 => (i + 1) % all.len(),
            _ => 0,
        };
        let list: Vec<(String, String)> = all[start..].iter().chain(all[..start].iter()).cloned().collect();
        // the film's length rules out a file made for another cut (0 while the video does not know it yet)
        let duration = video_element().map(|v| v.duration()).filter(|d| d.is_finite()).unwrap_or(0.0);
        let total = all.len();
        leptos::task::spawn_local(async move {
            let token = crate::services::account::ensure_fresh().await.map(|s| s.access_token).unwrap_or_default();
            let json = serde_json::to_string(&list.iter().map(|(url, lang)| {
                let auth = if url.starts_with(crate::config::RESOLVER_URL) { token.as_str() } else { "" };
                serde_json::json!({ "url": url, "lang": lang, "auth": auth })
            }).collect::<Vec<_>>()).unwrap_or_default();
            let loaded = subtitle_load(&json, duration).await.as_string().unwrap_or_default();
            // the choice may have changed while the file loaded
            if selected.get_untracked().as_deref() != Some(key.as_str()) {
                return;
            }
            if let Ok(l) = serde_json::from_str::<Loaded>(&loaded) {
                showing.set_value(Some((key.clone(), (start + l.index) % total)));
                cues.set(Some(std::sync::Arc::new(l.cues)));
            }
        });
    });

    // the same lines as the video's own text track, for Picture in Picture and a system's own full screen player (interop/subtitles.ts),
    // given again when the offset changes and taken away when subtitles are off
    Effect::new(move |_| {
        let d = delay.get();
        let json = cues.with(|c| c.as_ref().map(|list| serde_json::to_string(list.as_ref()).unwrap_or_default()).unwrap_or_default());
        subtitle_native_track(crate::keys::VIDEO_ELEMENT_ID, &json, d);
    });

    // ten times a second the video's own clock picks the line (the player's time signal moves four times a second)
    let tick = leptos::prelude::set_interval_with_handle(
        move || {
            let Some(list) = cues.get_untracked() else { return };
            let Some(video) = video_element() else { return };
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
    fn asks_the_viewers_languages_first() {
        assert!(wanted_languages(&["fr".into(), "pt-BR".into()]).starts_with("FR,BR_PT,EN,AR"));
        assert!(wanted_languages(&[]).starts_with("EN,AR"));
    }

    fn file(url: &str, label: &str) -> SubFile {
        SubFile { url: url.into(), lang: lang_tag::parse(label).unwrap() }
    }

    #[test]
    fn one_entry_per_language_viewers_first() {
        let files = vec![
            file("s1", "eng"), file("a", "EN"), file("b", "English SDH"), file("c", "BR_PT"), file("d", "pob"),
            file("e", "DE"), file("f", "AR"), file("g", "English - forced"), file("a", "EN"),
        ];
        let (tracks, cand) = build(&files, &["pt-BR".to_string(), "en".to_string()], |t| t.to_string());
        let keys: Vec<&str> = tracks.iter().map(|t| t.url.as_str()).collect();
        assert_eq!(keys, ["sub:pt-BR", "sub:en", "sub:ar", "sub:de"]);
        // the stream's own file first, the hearing impaired one last, the forced one left out, no file twice
        assert_eq!(cand["sub:en"].iter().map(|(u, _)| u.as_str()).collect::<Vec<_>>(), ["s1", "a", "b"]);
        assert_eq!(cand["sub:pt-BR"].len(), 2);
        assert_eq!(key_for(&cand, "pt").as_deref(), Some("sub:pt-BR"));
        assert_eq!(key_for(&cand, "fr"), None);
    }
}
