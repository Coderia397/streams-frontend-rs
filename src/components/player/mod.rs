// Autoplay and resilient streaming subsystem
pub mod timeline_scrubber;
pub mod controls;
pub mod frame;
pub mod open;
pub mod episode_drawer;
pub mod subtitle_drawer;
pub mod skip_button;
pub mod skip_detector;
pub mod rating_card;
pub mod subtitles;
pub mod website_overlays;

use leptos::prelude::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::KeyboardEvent;

use crate::components::hls_adapter::HlsPlayer;
use crate::services::resolver::{resolve_stream, PlayableKind, StreamResolveResult};
use crate::store::use_watch_store;

use controls::{ControlsState, PlayerControls};
use episode_drawer::EpisodeDrawer;
use subtitle_drawer::SubtitleDrawer;
use skip_button::SkipButton;
use rating_card::RatingCard;
use website_overlays::{Disc, PauseScreen, PlayerDiscs};
use crate::components::media::icons::app_icons::{AppIcon, Glyph};

/// The website's pause screen comes this long after the pointer last moved while paused (Netflix's: 10.1 s, note 126).
const PAUSE_SCREEN_MS: i32 = 10_000;

#[derive(Clone, Debug, PartialEq)]
enum PlayerState {
    Resolving,
    Playing {
        result: StreamResolveResult,
        active_source_idx: usize,
    },
    Error(String),
}

fn set_timeout_handle<F: FnOnce() + 'static>(cb: F, ms: i32) -> Option<i32> {
    let window = web_sys::window()?;
    let closure = Closure::once_into_js(cb);
    window
        .set_timeout_with_callback_and_timeout_and_arguments_0(closure.as_ref().unchecked_ref(), ms)
        .ok()
}

/// The source to try after `failed`: the next one in order that is not on the same host, and not another stream of the same provider
/// (a provider whose video host refuses this site, Cineby or Vidflix on 2026-10-25, lists five links that all fail the same way).
/// Ids of audio entries that are another source's language start here (below it: the tracks of the stream playing).
const AUDIO_ELSEWHERE: i32 = 1000;

/// The languages a source says its audio is in, as tags.
fn audio_tags(s: &crate::services::resolver::PlayableSource) -> Vec<String> {
    let listed: Vec<&str> = if s.audio_languages.is_empty() { s.audio.as_deref().filter(|a| *a != "multi").into_iter().collect() } else { s.audio_languages.iter().map(String::as_str).collect() };
    let mut tags: Vec<String> = Vec::new();
    for t in listed.into_iter().filter_map(crate::utils::lang_tag::parse).map(|l| l.tag) {
        if !tags.contains(&t) {
            tags.push(t);
        }
    }
    tags
}

pub(crate) fn next_source(sources: &[crate::services::resolver::PlayableSource], failed: usize) -> Option<usize> {
    let host = |u: &str| u.split("://").nth(1).and_then(|r| r.split('/').next()).unwrap_or("").to_ascii_lowercase();
    let bad = sources.get(failed)?;
    let (bad_host, bad_streamed) = (host(&bad.url), bad.kind != PlayableKind::Embed);
    (failed + 1..sources.len()).find(|&j| {
        let s = &sources[j];
        host(&s.url) != bad_host && !(bad_streamed && s.kind != PlayableKind::Embed && s.provider == bad.provider)
    })
}

fn clear_timeout_handle(id: Option<i32>) {
    if let Some(id) = id {
        if let Some(window) = web_sys::window() {
            window.clear_timeout_with_handle(id);
        }
    }
}

#[component]
pub fn StreamsPlayer(
    media_id: u32,
    title: String,
    is_tv: bool,
    season: Option<u32>,
    episode: Option<u32>,
    episode_title: Option<String>,
    year: Option<String>,
    poster_path: Option<String>,
    backdrop_path: Option<String>,
    orig_lang: Option<String>,
    on_close: Callback<()>,
    on_episode_change: Option<Callback<(u32, u32)>>,
) -> impl IntoView {
    let watch_store = use_watch_store();
    let player_container_ref = NodeRef::<leptos::html::Div>::new();
    let video_ref = NodeRef::<leptos::html::Video>::new();

    let (player_state, set_player_state) = signal(PlayerState::Resolving);
    let (is_playing, set_is_playing) = signal(true);
    let (is_buffering, set_is_buffering) = signal(false);
    let (is_muted, set_is_muted) = signal(false);
    let (volume, set_volume) = signal(1.0);
    let (current_time, set_current_time) = signal(0.0);
    let (duration, set_duration) = signal(0.0);
    let (speed, set_speed) = signal(1.0f64);
    let (buffered_fraction, set_buffered_fraction) = signal(0.0);
    let (is_idle, set_is_idle) = signal(false);
    // the phone's controls showing and its locked screen (controls/phone sets them; the skip buttons follow them)
    let phone_shown = RwSignal::new(true);
    let phone_locked = RwSignal::new(false);
    let form = crate::form::use_form();
    // the disc the last skip or play and pause leaves on the website's picture, numbered so a repeat brings a new one (website_overlays.rs)
    let (disc, set_disc) = signal(None::<(Disc, u32)>);
    let show_disc = move |d: Disc| set_disc.update(|v| *v = Some((d, v.map_or(0, |(_, n)| n.wrapping_add(1)))));
    // the website's pause screen, a while after the pointer stopped while paused
    let (pause_screen, set_pause_screen) = signal(false);
    let pause_timer = StoredValue::new(None::<i32>);
    let (pending_resume_time, set_pending_resume_time) = signal(0.0);
    // when and where the video was first seen playing sound without a picture (wall clock ms, video seconds)
    let audio_only_since = StoredValue::new(None::<(f64, f64)>);

    // Drawers
    let (is_episodes_open, set_is_episodes_open) = signal(false);
    let (is_subtitles_open, set_is_subtitles_open) = signal(false);
    let (selected_sub_url, set_selected_sub_url) = signal(None::<String>);
    let (sub_delay, set_sub_delay) = signal(0.0);
    let (sub_retry, set_sub_retry) = signal(0u32);
    // SubDL's files for this title (subtitles.rs); with the stream's own tracks they make one list, one entry per language
    let subdl_found = RwSignal::new(Vec::<subtitles::SubFile>::new());
    let prefs = crate::pages::settings::use_prefs();
    let profiles = crate::store::use_profile_store();
    // the profile's subtitle language: Some(tag), Some("off"), or None (follow: subtitles come on when the audio is not the viewer's language)
    let sub_pref = move || {
        profiles.active_profile_id.get_untracked().and_then(|id| prefs.profile(&id).subtitle_language).map(|p| {
            if p == "off" { p } else { crate::utils::lang_tag::parse(&p).map(|l| l.tag).unwrap_or(p) }
        })
    };
    let ui_tag = crate::utils::lang_tag::parse(&crate::i18n::language()).map(|l| l.tag).unwrap_or_else(|| "en".into());
    let orig_tag = orig_lang.as_deref().and_then(crate::utils::lang_tag::parse).map(|l| l.tag);
    // the languages that lead the list and are asked of SubDL first: the profile's, the interface's, the film's own
    let first_langs: Vec<String> = {
        let mut v: Vec<String> = Vec::new();
        for t in [sub_pref().filter(|p| p != "off"), Some(ui_tag.clone()), orig_tag.clone()].into_iter().flatten() {
            if !v.contains(&t) {
                v.push(t);
            }
        }
        v
    };
    let first_stored = StoredValue::new(first_langs.clone());
    // a choice in the player is kept as the profile's subtitle language (off included), as Netflix keeps it for the next title
    let choose_sub = Callback::new(move |key: Option<String>| {
        set_sub_retry.set(0);
        set_selected_sub_url.set(key.clone());
        let lang = match key.as_deref() {
            None => Some("off".to_string()),
            Some(k) => k.strip_prefix(subtitles::KEY).map(str::to_string),
        };
        if let (Some(lang), Some(id)) = (lang, profiles.active_profile_id.get_untracked()) {
            prefs.update_profile(&id, |p| p.subtitle_language = Some(lang));
        }
    });

    // Internal HLS & timers
    let hls_player = RwSignal::new_local(None::<std::rc::Rc<HlsPlayer>>);
    // the audio tracks of the stream (read each time the panel opens, since the stream lists them a moment after it starts)
    let audio_tracks = RwSignal::new(Vec::<(i32, String)>::new());
    let (audio_selected, set_audio_selected) = signal(0i32);
    // a language another source has (ids from AUDIO_ELSEWHERE): which source and which language; picking it moves to that source
    let audio_elsewhere = StoredValue::new(std::collections::HashMap::<i32, (usize, String)>::new());
    // after such a move: the moment to go on from, and the language to pick in the new stream once it lists its tracks
    let switch_resume = StoredValue::new(None::<f64>);
    let switch_audio = StoredValue::new(None::<String>);
    Effect::new(move |_| {
        if is_subtitles_open.get() {
            if let Some(p) = hls_player.get_untracked() {
                audio_tracks.set(p.audio_tracks());
                set_audio_selected.set(p.audio_track());
            }
        }
    });
    // the address of the stream attached now (a new resolve, even to the same address, clears it so the stream is attached again)
    let attached_url = StoredValue::new(None::<String>);
    let idle_timer = StoredValue::new(None::<i32>);
    let progress_timer = StoredValue::new(None::<i32>);
    let title_stored = StoredValue::new(title.clone());

    // Resolve Stream
    let do_resolve = Callback::new({
        let title_c = title.clone();
        let year_c = year.clone();
        let orig_lang_c = orig_lang.clone();
        move |force: bool| {
            set_player_state.set(PlayerState::Resolving);
            attached_url.set_value(None);
            let t = title_c.clone();
            let y = year_c.clone();
            let ol = orig_lang_c.clone();
            leptos::task::spawn_local(async move {
                match resolve_stream(media_id, is_tv, &t, y.as_deref(), season, episode, force, ol.as_deref()).await {
                    Ok(result) => {
                        set_player_state.set(PlayerState::Playing {
                            result,
                            active_source_idx: 0,
                        });
                    }
                    Err(e) => {
                        set_player_state.set(PlayerState::Error(e));
                    }
                }
            });
        }
    });

    // Initial resolve, with the stream library fetched at the same time (it was fetched only once the address was known)
    Effect::new(move |_| {
        crate::components::hls_adapter::hls_warm();
        do_resolve.run(false);
    });

    // the pause screen goes, and comes back PAUSE_SCREEN_MS after this if the video is still paused
    let arm_pause_screen = move || {
        if pause_screen.get_untracked() {
            set_pause_screen.set(false);
        }
        clear_timeout_handle(pause_timer.get_value());
        let t = if is_playing.get_untracked() {
            None
        } else {
            set_timeout_handle(move || {
                if !is_playing.get_untracked() {
                    set_pause_screen.set(true);
                }
            }, PAUSE_SCREEN_MS)
        };
        pause_timer.set_value(t);
    };
    Effect::new(move |_| {
        is_playing.track();
        arm_pause_screen();
    });

    // Idle timer reset
    let reset_idle_timer = move || {
        arm_pause_screen();
        set_is_idle.set(false);
        clear_timeout_handle(idle_timer.get_value());
        let t = set_timeout_handle(move || {
            set_is_idle.set(true);
        }, 2500);
        idle_timer.set_value(t);
    };

    let on_mouse_move = move |_| {
        reset_idle_timer();
    };

    // Attach stream when video element and player_state ready. The stream is attached once per address: the effect reads the saved progress, which changes every
    // few seconds while playing, and that must never attach the stream again (the video would reload and buffer again and again).
    Effect::new(move |_| {
        if let PlayerState::Playing { ref result, active_source_idx } = player_state.get() {
            if let Some(source) = result.sources.get(active_source_idx) {
                if source.kind != PlayableKind::Embed {
                    if let Some(video) = video_ref.get() {
                        let stream_url = &source.url;
                        if attached_url.get_value().as_deref() == Some(stream_url.as_str()) { return; }
                        attached_url.set_value(Some(stream_url.clone()));
                        let player = HlsPlayer::attach(video.clone(), stream_url);
                        hls_player.set(Some(std::rc::Rc::new(player)));

                        // Store resume time for when metadata/canplay is ready: the moment of a source switch, else the saved progress
                        let resume_t = switch_resume.get_value().take().unwrap_or_else(|| untrack(|| watch_store.get_resume_time(media_id, is_tv, season, episode)));
                        switch_resume.set_value(None);
                        if resume_t > 0.0 {
                            set_pending_resume_time.set(resume_t);
                        }

                        start_playing(video.clone(), set_is_muted, 2);
                        set_is_playing.set(true);
                    }
                }
            }
        }
    });

    // Progress persistence loop
    {
        let title_c = title.clone();
        let poster_c = poster_path.clone();
        let backdrop_c = backdrop_path.clone();
        let ep_title_c = episode_title.clone();

        Effect::new(move |_| {
            if is_playing.get() {
                clear_timeout_handle(progress_timer.get_value());
                let t_title = title_c.clone();
                let t_poster = poster_c.clone();
                let t_backdrop = backdrop_c.clone();
                let t_ep_title = ep_title_c.clone();

                let handle = set_timeout_handle(move || {
                    let cur = current_time.get_untracked();
                    let dur = duration.get_untracked();
                    if cur > 5.0 && dur > 0.0 {
                        watch_store.save_progress(
                            media_id,
                            is_tv,
                            t_title,
                            t_poster,
                            t_backdrop,
                            season,
                            episode,
                            t_ep_title,
                            cur,
                            dur,
                        );
                    }
                }, 4000);
                progress_timer.set_value(handle);
            }
        });
    }

    // Play/Pause Action
    let do_toggle_play = move || {
        reset_idle_timer();
        if let Some(video) = video_ref.get() {
            if video.paused() {
                let _ = video.play();
                set_is_playing.set(true);
                show_disc(Disc::Play);
            } else {
                let _ = video.pause();
                set_is_playing.set(false);
                show_disc(Disc::Pause);
            }
        }
    };

    // Seek Action
    let do_seek = move |target_secs: f64| {
        reset_idle_timer();
        if let Some(video) = video_ref.get() {
            let clamped = target_secs.clamp(0.0, duration.get());
            video.set_current_time(clamped);
            set_current_time.set(clamped);
        }
    };

    // Skip Relative Action
    let do_skip_by = move |delta_secs: f64| {
        let current = current_time.get();
        do_seek(current + delta_secs);
        show_disc(if delta_secs < 0.0 { Disc::Back } else { Disc::Forward });
    };

    // Mute Toggle
    let do_toggle_mute = move || {
        reset_idle_timer();
        if let Some(video) = video_ref.get() {
            let new_mute = !video.muted();
            video.set_muted(new_mute);
            set_is_muted.set(new_mute);
        }
    };

    // Set Volume
    let do_set_volume = move |vol: f64| {
        reset_idle_timer();
        let clamped = vol.clamp(0.0, 1.0);
        if let Some(video) = video_ref.get() {
            video.set_volume(clamped);
            if clamped == 0.0 {
                video.set_muted(true);
                set_is_muted.set(true);
            } else if video.muted() {
                video.set_muted(false);
                set_is_muted.set(false);
            }
            set_volume.set(clamped);
        }
    };

    // Playback speed (the phone's speed menu)
    let do_set_speed = move |v: f64| {
        if let Some(video) = video_ref.get() {
            video.set_playback_rate(v);
        }
        set_speed.set(v);
    };

    // Fullscreen Toggle
    let do_toggle_fullscreen = move || {
        reset_idle_timer();
        if let Some(win) = web_sys::window() {
            if let Some(doc) = win.document() {
                if doc.fullscreen_element().is_some() {
                    let _ = doc.exit_fullscreen();
                } else if let Some(el) = player_container_ref.get() {
                    let _ = el.request_fullscreen();
                }
            }
        }
    };

    // Video Element Event Handlers
    let on_time_update = move |_| {
        if let Some(video) = video_ref.get() {
            let cur = video.current_time();
            set_current_time.set(cur);
            let dur = video.duration();
            if !dur.is_nan() && dur > 0.0 {
                set_duration.set(dur);
                let frac = ((cur + 60.0) / dur).clamp(0.0, 1.0);
                set_buffered_fraction.set(frac);
            }

            // Sound without a picture for 3 s of playing means the codec is not supported here or the source is broken: the next source
            // plays. One time update without a picture is not enough, as it was: the width reads 0 for a moment while hls.js recovers
            // after a long seek, and that switched sources, so the video reloaded (and, before start_playing, went mute).
            let no_picture = cur > 2.0 && !video.paused() && video.video_width() == 0;
            let since = if no_picture { audio_only_since.get_value().or(Some((js_sys::Date::now(), cur))) } else { None };
            audio_only_since.set_value(since);
            if since.is_some_and(|(at, from)| js_sys::Date::now() - at > 3_000.0 && cur - from > 2.5) {
                audio_only_since.set_value(None);
                leptos::logging::warn!("Stream playing audio only with 0 video dimensions. Failing over to next source...");
                if let PlayerState::Playing { ref result, active_source_idx } = player_state.get_untracked() {
                    if let Some(next) = next_source(&result.sources, active_source_idx) {
                        set_player_state.set(PlayerState::Playing { result: result.clone(), active_source_idx: next });
                    }
                }
            }
        }
    };

    let on_waiting = move |_| set_is_buffering.set(true);
    // a seek starts the picture check again
    let on_seeking = move |_| audio_only_since.set_value(None);
    let on_canplay = move |_| {
        set_is_buffering.set(false);
        // after a move to another source for its language: pick that language once the stream lists its tracks
        if let Some(tag) = switch_audio.get_value() {
            for wait in [300, 1500] {
                let tag = tag.clone();
                set_timeout_handle(move || {
                    let Some(p) = hls_player.get_untracked() else { return };
                    let tracks = p.audio_tracks();
                    let found = tracks.iter().find(|(_, label)| crate::utils::lang_tag::parse(label).is_some_and(|l| crate::utils::lang_tag::base(&l.tag) == crate::utils::lang_tag::base(&tag)));
                    if let Some((i, _)) = found {
                        p.set_audio_track(*i);
                        set_audio_selected.set(*i);
                        audio_tracks.set(tracks.clone());
                        switch_audio.set_value(None);
                    }
                }, wait);
            }
        }
        let pending_t = pending_resume_time.get_untracked();
        if pending_t > 0.0 {
            if let Some(video) = video_ref.get() {
                video.set_current_time(pending_t);
                set_current_time.set(pending_t);
                set_pending_resume_time.set(0.0);
            }
        }
        if let Some(video) = video_ref.get() {
            if !video.paused() {
                set_is_playing.set(true);
            }
        }
    };
    let on_playing = move |_| {
        set_is_buffering.set(false);
        set_is_playing.set(true);
    };
    let on_pause = move |_| set_is_playing.set(false);

    let on_error = move |_| {
        leptos::logging::warn!("Video playback error encountered on current stream source, attempting failover...");
        if let PlayerState::Playing { ref result, active_source_idx } = player_state.get_untracked() {
            if let Some(next) = next_source(&result.sources, active_source_idx) {
                set_player_state.set(PlayerState::Playing { result: result.clone(), active_source_idx: next });
            } else {
                set_player_state.set(PlayerState::Error(
                    crate::i18n::t("player-error-no-source"),
                ));
            }
        }
    };

    // Keyboard shortcuts, for the player's life. The browser's and the system's own shortcuts (Cmd or Ctrl with a key, Alt) and typing in a
    // field are left alone, and Space on a focused button is the button's (it toggled twice before: the button, then this).
    crate::utils::listen::listen_window(leptos::ev::keydown, move |e: KeyboardEvent| {
        if e.meta_key() || e.ctrl_key() || e.alt_key() {
            return;
        }
        if let Some(el) = e.target().and_then(|t| t.dyn_into::<web_sys::HtmlElement>().ok()) {
            let tag = el.tag_name();
            if matches!(tag.as_str(), "INPUT" | "TEXTAREA" | "SELECT") || el.is_content_editable() {
                return;
            }
            if e.key() == " " && tag == "BUTTON" {
                return;
            }
        }
        let key = e.key();
        match key.as_str() {
            " " | "k" | "K" => {
                e.prevent_default();
                do_toggle_play();
            }
            "ArrowLeft" | "j" | "J" => {
                e.prevent_default();
                do_skip_by(-10.0);
            }
            "ArrowRight" | "l" | "L" => {
                e.prevent_default();
                do_skip_by(10.0);
            }
            "ArrowUp" => {
                e.prevent_default();
                do_set_volume(volume.get_untracked() + 0.10);
            }
            "ArrowDown" => {
                e.prevent_default();
                do_set_volume(volume.get_untracked() - 0.10);
            }
            "m" | "M" => {
                e.prevent_default();
                do_toggle_mute();
            }
            "f" | "F" => {
                e.prevent_default();
                do_toggle_fullscreen();
            }
            _ => {}
        }
    });

    on_cleanup(move || {
        clear_timeout_handle(idle_timer.get_value());
        clear_timeout_handle(progress_timer.get_value());
        clear_timeout_handle(pause_timer.get_value());
    });

    // Subtitle Display Logic
    // one list: the stream's own tracks first, then SubDL's files, one entry per language (subtitles::build)
    let subtitle_list = Memo::new(move |_| {
        let mut files = if let PlayerState::Playing { ref result, .. } = player_state.get() { subtitles::stream_files(&result.subtitles) } else { vec![] };
        files.extend(subdl_found.get());
        first_stored.with_value(|first| subtitles::build(&files, first, crate::i18n::language_name))
    });
    let current_subtitle_tracks = move || subtitle_list.with(|l| l.0.clone());
    let subtitle_files = Signal::derive(move || subtitle_list.with(|l| l.1.clone()));

    // SubDL's list, asked once the stream is found; the profile's subtitle language is switched on when it is there
    Effect::new(move |done: Option<bool>| {
        if done == Some(true) {
            return true;
        }
        if !matches!(player_state.get(), PlayerState::Playing { .. }) {
            return false;
        }
        let preferred = sub_pref();
        let first = first_stored.get_value();
        let (ui, orig) = (ui_tag.clone(), orig_tag.clone());
        leptos::task::spawn_local(async move {
            let found = subtitles::subdl_files(media_id, is_tv, season, episode, &first).await;
            subdl_found.set(found);
            if selected_sub_url.get_untracked().is_some() {
                return;
            }
            // the profile's language when it is there; with no choice saved (follow), the viewer's language when the film is in another
            let wanted = match preferred.as_deref() {
                Some("off") => None,
                Some(tag) => Some(tag.to_string()),
                None => orig.filter(|o| crate::utils::lang_tag::base(o) != crate::utils::lang_tag::base(&ui)).map(|_| ui),
            };
            if let Some(key) = wanted.and_then(|w| subtitle_files.with_untracked(|c| subtitles::key_for(c, &w))) {
                set_selected_sub_url.set(Some(key));
            }
        });
        true
    });

    // the Audio list: the tracks of the stream playing, named in the viewer's language, then each language another source has (once);
    // picking one of those moves to that source at the same moment
    let audio_options = Signal::derive(move || {
        let mut out: Vec<(i32, String)> = Vec::new();
        let mut have: Vec<String> = Vec::new();
        for (i, label) in audio_tracks.get() {
            match crate::utils::lang_tag::parse(&label) {
                Some(l) => {
                    out.push((i, crate::i18n::language_name(&l.tag)));
                    have.push(l.tag);
                }
                None => out.push((i, label)),
            }
        }
        let mut elsewhere = std::collections::HashMap::new();
        if let PlayerState::Playing { ref result, active_source_idx } = player_state.get() {
            if let Some(cur) = result.sources.get(active_source_idx) {
                for t in audio_tags(cur) {
                    if !have.iter().any(|h| crate::utils::lang_tag::base(h) == crate::utils::lang_tag::base(&t)) {
                        have.push(t);
                    }
                }
            }
            let mut id = AUDIO_ELSEWHERE;
            for (j, src) in result.sources.iter().enumerate().filter(|(j, _)| *j != active_source_idx) {
                for t in audio_tags(src) {
                    if have.iter().any(|h| crate::utils::lang_tag::base(h) == crate::utils::lang_tag::base(&t)) {
                        continue;
                    }
                    out.push((id, crate::i18n::language_name(&t)));
                    elsewhere.insert(id, (j, t.clone()));
                    have.push(t);
                    id += 1;
                }
            }
        }
        audio_elsewhere.set_value(elsewhere);
        out
    });
    let choose_audio = Callback::new(move |i: i32| {
        if i >= AUDIO_ELSEWHERE {
            let Some((j, tag)) = audio_elsewhere.with_value(|m| m.get(&i).cloned()) else { return };
            if let PlayerState::Playing { ref result, .. } = player_state.get_untracked() {
                // an embedded player keeps its own moment and tracks: only a stream played here goes on from the same moment
                if result.sources.get(j).is_some_and(|s| s.kind != PlayableKind::Embed) {
                    switch_resume.set_value(Some(current_time.get_untracked()));
                    switch_audio.set_value(Some(tag));
                }
                set_player_state.set(PlayerState::Playing { result: result.clone(), active_source_idx: j });
            }
            return;
        }
        if let Some(p) = hls_player.get_untracked() { p.set_audio_track(i); }
        set_audio_selected.set(i);
    });

    // the title block as on Netflix: the show in bold, then "E3" and the episode's name; a film shows its title only
    let ep_name = LocalResource::new(move || async move {
        if is_tv && media_id > 0 {
            crate::services::tmdb::fetch_season_details(media_id, season.unwrap_or(1)).await.ok()
                .and_then(|d| d.episodes.into_iter().find(|e| e.episode_number == episode.unwrap_or(1)).map(|e| e.name))
        } else { None }
    });
    // the summary on the website's pause screen: the episode's, or the film's
    let synopsis_res = LocalResource::new(move || async move {
        if media_id == 0 {
            return None;
        }
        if is_tv {
            crate::services::tmdb::fetch_season_details(media_id, season.unwrap_or(1)).await.ok()
                .and_then(|d| d.episodes.into_iter().find(|e| e.episode_number == episode.unwrap_or(1)).map(|e| e.overview))
        } else {
            crate::services::tmdb::fetch_details(media_id, false).await.ok().map(|d| d.overview)
        }
    });
    let synopsis_sig = Signal::derive(move || synopsis_res.get().flatten());
    // the stream playing here (not an embedded player), for the timeline's pictures
    let stream_url = Signal::derive(move || match player_state.get() {
        PlayerState::Playing { ref result, active_source_idx } => result.sources.get(active_source_idx).filter(|s| s.kind != PlayableKind::Embed).map(|s| s.url.clone()),
        _ => None,
    });
    let is_hls = Signal::derive(move || match player_state.get() {
        PlayerState::Playing { ref result, active_source_idx } => {
            result.sources.get(active_source_idx).map_or(false, |s| s.kind == PlayableKind::Hls)
        }
        _ => false,
    });
    let episode_title_stored = StoredValue::new(episode_title.clone());
    let subtitle_header_text = move || {
        if is_tv {
            match episode_title.clone().or_else(|| ep_name.get().flatten()) {
                Some(n) if !n.is_empty() => format!("E{}  {}", episode.unwrap_or(1), n),
                _ => format!("E{}", episode.unwrap_or(1)),
            }
        } else {
            String::new()
        }
    };

    let next_ep = LocalResource::new(move || async move {
        if is_tv && media_id > 0 {
            let n = episode.unwrap_or(1) + 1;
            crate::services::tmdb::fetch_season_details(media_id, season.unwrap_or(1)).await.ok()
                .and_then(|d| d.episodes.into_iter().find(|e| e.episode_number == n))
                .map(|e| (n, e.name.clone(), e.overview.clone(), e.still_url("w300")))
        } else { None }
    });
    let next_sig = Signal::derive(move || next_ep.get().flatten());
    // the guessed skip marks of this video (ratios of its length); an episode is the last of its season when no episode follows it in the season's list
    // films have no skip buttons (no intro or recap to skip, no next episode): the engine is not asked
    let active_cues = RwSignal::new(None::<std::sync::Arc<Vec<subtitles::Cue>>>);
    let on_cues_change = Callback::new(move |cues: Option<std::sync::Arc<Vec<subtitles::Cue>>>| {
        active_cues.set(cues);
    });

    // background English subtitle cues loaded for skip detection even when viewer subtitles are off
    let bg_english_cues = RwSignal::new(None::<std::sync::Arc<Vec<subtitles::Cue>>>);
    Effect::new(move |loaded: Option<bool>| {
        if loaded == Some(true) || !is_tv {
            return true;
        }
        let d = duration.get();
        if d <= 0.0 || d.is_nan() {
            return false;
        }
        let candidates = subtitle_files.get();
        let Some(en_key) = subtitles::key_for(&candidates, "en") else { return false };
        // If viewer already has the English subtitle track selected, active_cues handles detection;
        // return false so turning subtitles off later will re-trigger this effect and load in the background
        if selected_sub_url.get().as_ref() == Some(&en_key) {
            return false;
        }
        let Some(list) = candidates.get(&en_key).cloned() else { return false };
        if list.is_empty() {
            return false;
        }
        leptos::task::spawn_local(async move {
            if let Some(cues) = subtitles::load_cues(&list, d).await {
                bg_english_cues.set(Some(std::sync::Arc::new(cues)));
            }
        });
        true
    });

    let marks_res = LocalResource::new(move || async move {
        if !is_tv {
            return crate::services::ai_engine::SkipMarks::default();
        }
        let last = if is_tv && media_id > 0 {
            crate::services::tmdb::fetch_season_details(media_id, season.unwrap_or(1)).await.ok()
                .map(|d| d.episodes.iter().all(|e| e.episode_number <= episode.unwrap_or(1))).unwrap_or(false)
        } else { false };
        crate::services::ai_engine::fetch_skip_marks(is_tv, media_id, episode, last).await
    });

    let marks_with_origin = Memo::new(move |_| {
        let d = duration.get();
        let prior = marks_res.get();
        let candidates = subtitle_files.get();
        let en_key = subtitles::key_for(&candidates, "en");
        let is_active_english = selected_sub_url.get().is_some() && selected_sub_url.get() == en_key;
        let (cues, delay) = if is_active_english {
            (active_cues.get(), sub_delay.get())
        } else if let Some(bg_cues) = bg_english_cues.get() {
            (Some(bg_cues), 0.0)
        } else {
            (active_cues.get(), sub_delay.get())
        };
        match cues {
            Some(c) => skip_detector::detect_marks_from_cues(&c, d, delay, is_tv, prior.as_ref()),
            None => {
                let marks = prior.unwrap_or_default();
                skip_detector::SkipMarksWithOrigin {
                    intro_origin: marks.intro.map(|_| skip_detector::MarkOrigin::Engine),
                    recap_origin: marks.recap.map(|_| skip_detector::MarkOrigin::Engine),
                    credits_origin: marks.credits.map(|_| skip_detector::MarkOrigin::Engine),
                    intro_strength: marks.intro.map(|_| skip_detector::SignalStrength::Medium),
                    recap_strength: marks.recap.map(|_| skip_detector::SignalStrength::Medium),
                    credits_strength: marks.credits.map(|_| skip_detector::SignalStrength::Medium),
                    marks,
                }
            }
        }
    });

    Effect::new(move |_| {
        let m = marks_with_origin.get();
        if m.marks.intro.is_some() || m.marks.recap.is_some() || m.marks.credits.is_some() {
            #[cfg(target_arch = "wasm32")]
            web_sys::console::log_1(&wasm_bindgen::JsValue::from_str(&format!(
                "[SkipMarks] intro={:?} ({:?}, {:?}), recap={:?} ({:?}, {:?}), credits={:?} ({:?}, {:?})",
                m.marks.intro, m.intro_origin, m.intro_strength,
                m.marks.recap, m.recap_origin, m.recap_strength,
                m.marks.credits, m.credits_origin, m.credits_strength,
            )));
        }
    });

    let marks_sig = Signal::derive(move || marks_with_origin.with(|m| m.marks.clone()));
    let subtitle_sig = Signal::derive(subtitle_header_text);
    let episode_name_sig = Signal::derive(move || episode_title_stored.get_value().or_else(|| ep_name.get().flatten()).filter(|n| !n.is_empty()));

    view! {
        <div
            node_ref=player_container_ref
            on:mousemove=on_mouse_move
            class="relative w-full h-full bg-void overflow-hidden select-none"
        >
            {move || match player_state.get() {
                PlayerState::Resolving => view! {
                    <div class="absolute inset-0 flex items-center justify-center bg-void z-30" role="status" aria-label=crate::i18n::t("mod-finding-optimal-stream")>
                        <div class="aspect-square w-[4vw] animate-spin rounded-full border-[3px] border-transparent border-s-brand"></div>
                    </div>
                }.into_any(),

                PlayerState::Error(err_msg) => view! {
                    <div class="absolute inset-0 flex flex-col items-center justify-center bg-void z-30 text-center px-6">
                        <div class="w-16 h-16 rounded-sheet bg-ink/10 flex items-center justify-center mb-4 text-brand text-3xl">
                            <i class="ph ph-warning-circle"></i>
                        </div>
                        <h3 class="text-xl font-bold text-ink mb-2">{crate::i18n::t("player-unavailable")}</h3>
                        <p class="text-ink/60 text-sm max-w-md mb-6">{err_msg}</p>
                        <div class="flex items-center gap-4">
                            <button
                                on:click=move |_| do_resolve.run(true)
                                class="btn-brand px-6 py-2.5 text-sm"
                            >
                                {crate::i18n::t("mod-retry")}
                            </button>
                            <button
                                on:click=move |_| on_close.run(())
                                class="px-6 py-2.5 bg-ink/15 hover:bg-ink/25 text-ink font-semibold rounded-button transition-colors text-sm"
                            >
                                {crate::i18n::t("controls-overlay-back-to-browse")}
                            </button>
                        </div>
                    </div>
                }.into_any(),

                PlayerState::Playing { result, active_source_idx } => {
                    let source = result.sources.get(active_source_idx).cloned();
                    let is_embed = source.as_ref().map_or(false, |s| s.kind == PlayableKind::Embed);

                    view! {
                        <>
                            {if is_embed {
                                let embed_url = source.as_ref().map(|s| s.url.clone()).unwrap_or_default();
                                view! {
                                    <div class="w-full h-full relative">
                                        <iframe
                                            src=embed_url
                                            class="w-full h-full border-0"
                                            allow="autoplay; fullscreen; encrypted-media"
                                            allowfullscreen=true
                                        />
                                        // Top back bar overlay for embed fallback
                                        <div class="absolute top-4 left-4 z-30">
                                            <button
                                                on:click=move |_| on_close.run(())
                                                class="flex items-center gap-2 px-4 py-2 rounded-button bg-void/80 hover:bg-void text-ink text-sm font-semibold backdrop-blur-md shadow-lg"
                                            >
                                                <AppIcon glyph=Glyph::Back class="text-lg" />
                                                <span>{crate::i18n::t("player-back")}</span>
                                            </button>
                                        </div>
                                    </div>
                                }.into_any()
                            } else {
                                view! {
                                    <div class="relative w-full h-full flex items-center justify-center bg-void">
                                        <video
                                            node_ref=video_ref
                                            id=crate::keys::VIDEO_ELEMENT_ID
                                            on:timeupdate=on_time_update
                                            on:waiting=on_waiting
                                            on:seeking=on_seeking
                                            on:canplay=on_canplay
                                            on:playing=on_playing
                                            on:pause=on_pause
                                            on:error=on_error
                                            // a click reaches the video only while the website's controls are hidden (they cover it when shown, and play or
                                            // pause on a click; the phone's controls catch every tap): it brings the controls back, as on Netflix
                                            on:click=move |_| reset_idle_timer()
                                            class="w-full h-full object-contain cursor-pointer"
                                            playsinline=true
                                        />

                                        // the subtitle line, above the picture and under the controls
                                        <subtitles::SubtitleOverlay
                                            selected=selected_sub_url
                                            candidates=subtitle_files
                                            delay=sub_delay
                                            lifted=Signal::derive(move || !is_idle.get())
                                            retry=sub_retry
                                            on_cues_change=on_cues_change
                                        />

                                        // the controls of the form in use (controls/: website.rs, phone.rs)
                                        <PlayerControls state=ControlsState {
                                            title: title_stored,
                                            subtitle: subtitle_sig,
                                            next_info: next_sig,
                                            episode: if is_tv { Some((season.unwrap_or(1), episode.unwrap_or(1))) } else { None },
                                            episode_name: episode_name_sig,
                                            media_id,
                                            is_tv,
                                            is_playing,
                                            is_buffering,
                                            is_muted,
                                            volume,
                                            current_time,
                                            duration,
                                            buffered_fraction,
                                            is_idle,
                                            phone_shown,
                                            phone_locked,
                                            speed,
                                            on_set_speed: Callback::new(move |v| do_set_speed(v)),
                                            on_toggle_play: Callback::new(move |_| do_toggle_play()),
                                            on_seek: Callback::new(move |t| do_seek(t)),
                                            on_skip_by: Callback::new(move |d| do_skip_by(d)),
                                            on_toggle_mute: Callback::new(move |_| do_toggle_mute()),
                                            on_set_volume: Callback::new(move |v| do_set_volume(v)),
                                            on_toggle_episodes: Callback::new(move |_| set_is_episodes_open.update(|o| *o = !*o)),
                                            on_toggle_subtitles: Callback::new(move |_| set_is_subtitles_open.update(|o| *o = !*o)),
                                            on_next_episode: on_episode_change.map(|cb| Callback::new(move |_| {
                                                let next_ep = episode.unwrap_or(1) + 1;
                                                let curr_s = season.unwrap_or(1);
                                                cb.run((curr_s, next_ep));
                                            })),
                                            on_toggle_fullscreen: Callback::new(move |_| do_toggle_fullscreen()),
                                            on_back: on_close,
                                            subtitles: Signal::derive(current_subtitle_tracks),
                                            selected_sub: selected_sub_url,
                                            audio_tracks: audio_options,
                                            audio_selected: audio_selected.into(),
                                            on_select_audio: choose_audio,
                                            on_select_sub: choose_sub,
                                            on_refresh_audio: Callback::new(move |_| {
                                                if let Some(p) = hls_player.get_untracked() {
                                                    audio_tracks.set(p.audio_tracks());
                                                    set_audio_selected.set(p.audio_track());
                                                }
                                            }),
                                            on_select_episode: on_episode_change,
                                            stream_url,
                                            is_hls,
                                        } />

                                        // the age rating card, once at the start of a play
                                        <RatingCard media_id=media_id is_tv=is_tv season=season episode=episode current_time=current_time />

                                        // Skip Intro Button
                                        <SkipButton
                                            current_time=current_time
                                            duration=duration
                                            is_tv=is_tv
                                            marks=marks_sig
                                            lifted=Signal::derive(move || if form.get().is_phone() { phone_shown.get() && !phone_locked.get() } else { !is_idle.get() })
                                            hidden=phone_locked.into()
                                            on_skip_to=Callback::new(move |t| do_seek(t))
                                            on_next_episode=Callback::new(move |_| {
                                                if let Some(cb) = on_episode_change {
                                                    let next_ep = episode.unwrap_or(1) + 1;
                                                    let curr_s = season.unwrap_or(1);
                                                    cb.run((curr_s, next_ep));
                                                }
                                            })
                                        />

                                        // the website's drawers; the phone's menus are inside its controls (controls/phone)
                                        <crate::form::ByForm
                                            phone=|| ()
                                            desktop=move || view! {
                                        // the discs a skip or a play and pause leaves on the picture, and the pause screen (website_overlays.rs)
                                        <PlayerDiscs disc=disc />
                                        <PauseScreen
                                            shown=Signal::derive(move || {
                                                pause_screen.get() && !is_episodes_open.get() && !is_subtitles_open.get()
                                                    && duration.get() > 0.0 && current_time.get() < duration.get() - 2.0
                                            })
                                            title=title_stored
                                            season=season
                                            episode=episode
                                            episode_name=episode_name_sig
                                            synopsis=synopsis_sig
                                        />
                                        // Episode Drawer
                                        <EpisodeDrawer
                                            series_id=media_id
                                            current_season=season.unwrap_or(1)
                                            current_episode=episode.unwrap_or(1)
                                            is_open=is_episodes_open
                                            on_close=Callback::new(move |_| set_is_episodes_open.set(false))
                                            on_select_episode=Callback::new(move |(s, e)| {
                                                if let Some(cb) = on_episode_change {
                                                    cb.run((s, e));
                                                }
                                            })
                                        />

                                        // Subtitle Drawer
                                        <SubtitleDrawer
                                            is_open=is_subtitles_open
                                            subtitles=Signal::derive(current_subtitle_tracks)
                                            selected_sub_url=selected_sub_url
                                            sub_delay=sub_delay
                                            audio_tracks=audio_options
                                            audio_selected=audio_selected.into()
                                            on_select_audio=choose_audio
                                            on_select_sub=choose_sub
                                            on_change_delay=Callback::new(move |d| set_sub_delay.set(d))
                                            has_other_version=Signal::derive(move || selected_sub_url.get().is_some_and(|k| subtitle_files.with(|c| c.get(&k).is_some_and(|l| l.len() > 1))))
                                            on_other_version=Callback::new(move |_| set_sub_retry.update(|n| *n += 1))
                                            on_close=Callback::new(move |_| set_is_subtitles_open.set(false))
                                        />
                                            }
                                        />
                                    </div>
                                }.into_any()
                            }}
                        </>
                    }.into_any()
                }
            }}
        </div>
    }
}

/// Plays `video` as a stream starts. A refusal means one of two things. The browser wants a tap before sound (NotAllowedError): the
/// video plays muted, and on the phone, which has no mute button, the next tap on the video turns the sound on. Or a newer load replaced
/// this one (AbortError: hls.js sets the source after the call when the element already held a stream, at the next episode or a switch
/// to another source): the video plays once the new source can, up to `retries` times. Muting on every refusal, as before, muted the
/// next episode and every source switch.
fn start_playing(video: web_sys::HtmlVideoElement, set_is_muted: WriteSignal<bool>, retries: u8) {
    let Ok(promise) = video.play() else { return };
    let v = video.clone();
    // a one shot closure handed to JS, freed after it runs
    let on_reject = Closure::once(move |err: JsValue| {
        let name = js_sys::Reflect::get(&err, &"name".into()).ok().and_then(|n| n.as_string()).unwrap_or_default();
        if name == "NotAllowedError" {
            leptos::logging::warn!("Sound needs a tap first: playing muted");
            v.set_muted(true);
            set_is_muted.set(true);
            let _ = v.play();
        } else if retries > 0 {
            if v.ready_state() >= web_sys::HtmlMediaElement::HAVE_FUTURE_DATA {
                start_playing(v, set_is_muted, retries - 1);
            } else {
                let again = v.clone();
                let on_ready = Closure::once_into_js(move || start_playing(again, set_is_muted, retries - 1));
                let once = web_sys::AddEventListenerOptions::new();
                once.set_once(true);
                let _ = v.add_event_listener_with_callback_and_add_event_listener_options("canplay", on_ready.unchecked_ref(), &once);
            }
        }
    });
    let _ = promise.catch(&on_reject);
    on_reject.forget();
}

#[cfg(test)]
mod failover_tests {
    use super::*;
    use crate::services::resolver::PlayableSource;

    fn src(url: &str, provider: &str, kind: PlayableKind) -> PlayableSource {
        PlayableSource { url: url.into(), quality: "auto".into(), provider: provider.into(), kind, referer: None, audio: None, audio_languages: vec![], is_original: None, is_multi_audio: None, browser_ok: None }
    }

    #[test]
    fn a_failed_host_and_its_providers_streams_are_skipped() {
        let list = vec![
            src("https://v.lumenbox.top/a/2160", "Cineby", PlayableKind::Hls),
            src("https://v.lumenbox.top/a/1080", "Cineby", PlayableKind::Hls),
            src("https://cno.to/api/s/21/movie/1", "Cineby", PlayableKind::Hls),
            src("https://player.vidzee.wtf/embed/1", "HydraHD", PlayableKind::Embed),
            src("https://vidlink.pro/movie/1", "HydraHD", PlayableKind::Embed),
        ];
        assert_eq!(next_source(&list, 0), Some(3));
        // an embed that fails moves to the next embed of the same provider on another host
        assert_eq!(next_source(&list, 3), Some(4));
        assert_eq!(next_source(&list, 4), None);
    }

    #[test]
    fn audio_languages_of_a_source() {
        let mut s = src("https://x/1", "VixSrc", PlayableKind::Hls);
        s.audio_languages = vec!["en".into(), "ita".into()];
        assert_eq!(audio_tags(&s), ["en", "it"]);
        let mut s = src("https://x/2", "HydraHD", PlayableKind::Embed);
        s.audio = Some("ru".into());
        assert_eq!(audio_tags(&s), ["ru"]);
        s.audio = Some("multi".into());
        assert!(audio_tags(&s).is_empty());
    }
}
