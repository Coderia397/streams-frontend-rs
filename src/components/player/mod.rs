// Autoplay and resilient streaming subsystem
pub mod timeline_scrubber;
pub mod controls;
pub mod frame;
pub mod open;
pub mod episode_drawer;
pub mod subtitle_drawer;
pub mod skip_button;
pub mod rating_card;

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
    let (pending_resume_time, set_pending_resume_time) = signal(0.0);
    // when and where the video was first seen playing sound without a picture (wall clock ms, video seconds)
    let audio_only_since = StoredValue::new(None::<(f64, f64)>);

    // Drawers
    let (is_episodes_open, set_is_episodes_open) = signal(false);
    let (is_subtitles_open, set_is_subtitles_open) = signal(false);
    let (selected_sub_url, set_selected_sub_url) = signal(None::<String>);
    let (sub_delay, set_sub_delay) = signal(0.0);

    // Internal HLS & timers
    let hls_player = RwSignal::new_local(None::<std::rc::Rc<HlsPlayer>>);
    // the audio tracks of the stream (read each time the panel opens, since the stream lists them a moment after it starts)
    let audio_tracks = RwSignal::new(Vec::<(i32, String)>::new());
    let (audio_selected, set_audio_selected) = signal(0i32);
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

    // Initial resolve
    Effect::new(move |_| {
        do_resolve.run(false);
    });

    // Idle timer reset
    let reset_idle_timer = move || {
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

                        // Store resume time for when metadata/canplay is ready
                        let resume_t = untrack(|| watch_store.get_resume_time(media_id, is_tv, season, episode));
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
            } else {
                let _ = video.pause();
                set_is_playing.set(false);
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
                    if active_source_idx + 1 < result.sources.len() {
                        set_player_state.set(PlayerState::Playing {
                            result: result.clone(),
                            active_source_idx: active_source_idx + 1,
                        });
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
            if active_source_idx + 1 < result.sources.len() {
                set_player_state.set(PlayerState::Playing {
                    result: result.clone(),
                    active_source_idx: active_source_idx + 1,
                });
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
    });

    // Subtitle Display Logic
    let current_subtitle_tracks = move || {
        if let PlayerState::Playing { ref result, .. } = player_state.get() {
            result.subtitles.clone()
        } else {
            vec![]
        }
    };

    // the title block as on Netflix: the show in bold, then "E3" and the episode's name; a film shows its title only
    let ep_name = LocalResource::new(move || async move {
        if is_tv && media_id > 0 {
            crate::services::tmdb::fetch_season_details(media_id, season.unwrap_or(1)).await.ok()
                .and_then(|d| d.episodes.into_iter().find(|e| e.episode_number == episode.unwrap_or(1)).map(|e| e.name))
        } else { None }
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
    let marks_res = LocalResource::new(move || async move {
        let last = if is_tv && media_id > 0 {
            crate::services::tmdb::fetch_season_details(media_id, season.unwrap_or(1)).await.ok()
                .map(|d| d.episodes.iter().all(|e| e.episode_number <= episode.unwrap_or(1))).unwrap_or(false)
        } else { false };
        crate::services::ai_engine::fetch_skip_marks(is_tv, media_id, episode, last).await
    });
    let marks_sig = Signal::derive(move || marks_res.get().unwrap_or_default());
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
                                                <i class="ph ph-arrow-left text-lg"></i>
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
                                            on:click=move |_| do_toggle_play()
                                            class="w-full h-full object-contain cursor-pointer"
                                            playsinline=true
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
                                            audio_tracks: audio_tracks.into(),
                                            audio_selected: audio_selected.into(),
                                            on_select_audio: Callback::new(move |i| {
                                                if let Some(p) = hls_player.get_untracked() { p.set_audio_track(i); }
                                                set_audio_selected.set(i);
                                            }),
                                            on_select_sub: Callback::new(move |u| set_selected_sub_url.set(u)),
                                            on_refresh_audio: Callback::new(move |_| {
                                                if let Some(p) = hls_player.get_untracked() {
                                                    audio_tracks.set(p.audio_tracks());
                                                    set_audio_selected.set(p.audio_track());
                                                }
                                            }),
                                            on_select_episode: on_episode_change,
                                        } />

                                        // the age rating card, once at the start of a play
                                        <RatingCard media_id=media_id is_tv=is_tv season=season episode=episode current_time=current_time />

                                        // Skip Intro Button
                                        <SkipButton
                                            current_time=current_time
                                            duration=duration
                                            is_tv=is_tv
                                            marks=marks_sig
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
                                            audio_tracks=audio_tracks.into()
                                            audio_selected=audio_selected.into()
                                            on_select_audio=Callback::new(move |i| {
                                                if let Some(p) = hls_player.get_untracked() { p.set_audio_track(i); }
                                                set_audio_selected.set(i);
                                            })
                                            on_select_sub=Callback::new(move |u| set_selected_sub_url.set(u))
                                            on_change_delay=Callback::new(move |d| set_sub_delay.set(d))
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
