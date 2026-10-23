//! The billboard on tablets and computers (note 114).
use leptos::prelude::*;
use wasm_bindgen::prelude::*;
use crate::services::tmdb::{fetch_movie_logo, fetch_videos, fetch_details, MediaItem};
use crate::store::use_ui_store;
use crate::components::media::movie_card_badges::RatingBadge;
use crate::components::media::action_buttons::{PlayPillButton, MoreInfoPillButton, MuteReplayButton};
use crate::components::trailer_player::TrailerPlayer;

fn set_timeout_ms<F: FnOnce() + 'static>(cb: F, ms: i32) -> Option<i32> {
    let window = web_sys::window()?;
    let closure = Closure::once_into_js(cb);
    window
        .set_timeout_with_callback_and_timeout_and_arguments_0(closure.as_ref().unchecked_ref(), ms)
        .ok()
}

fn clear_timeout_id(id: Option<i32>) {
    if let Some(id) = id {
        if let Some(window) = web_sys::window() {
            window.clear_timeout_with_handle(id);
        }
    }
}

#[component]
fn MegaphoneBadgeIcon() -> impl IntoView {
    view! {
        <svg
            class="w-[15px] h-[15px] flex-shrink-0 drop-shadow-accent-pink"
            viewBox="0 0 20 20"
            fill="none"
            xmlns="http://www.w3.org/2000/svg"
        >
            <defs>
                <linearGradient id="megaGrad" x1="0%" y1="0%" x2="100%" y2="100%">
                    <stop offset="0%" style="stop-color: var(--color-accent-alt)" />
                    <stop offset="60%" stop-color="#ff1a40" />
                    <stop offset="100%" stop-color="#ff6b00" />
                </linearGradient>
            </defs>
            <path
                d="M4 11.5V8.5C4 7.95 4.45 7.5 5 7.5H7L13.2 3.4C13.8 3.0 14.6 3.4 14.6 4.1V15.9C14.6 16.6 13.8 17.0 13.2 16.6L7 12.5H5C4.45 12.5 4 12.05 4 11.5Z"
                fill="url(#megaGrad)"
            />
            <path
                d="M6 12.5V15.5C6 16.05 6.45 16.5 7 16.5H7.5C8.05 16.5 8.5 16.05 8.5 15.5V12.5"
                fill="url(#megaGrad)"
            />
            <path
                d="M16 7.5C16.8 8.8 16.8 11.2 16 12.5"
                stroke="url(#megaGrad)"
                stroke-width="1.8"
                stroke-linecap="round"
            />
        </svg>
    }
}

#[component]
fn ClapperboardBadgeIcon() -> impl IntoView {
    view! {
        <svg
            class="w-[15px] h-[15px] flex-shrink-0 drop-shadow-accent-violet"
            viewBox="0 0 20 20"
            fill="none"
            xmlns="http://www.w3.org/2000/svg"
        >
            <defs>
                <linearGradient id="clapGrad" x1="0%" y1="0%" x2="100%" y2="100%">
                    <stop offset="0%" style="stop-color: var(--color-accent-alt)" />
                    <stop offset="50%" stop-color="#9b5de5" />
                    <stop offset="100%" stop-color="#7209b7" />
                </linearGradient>
            </defs>
            <rect x="2.5" y="4" width="15" height="3.5" rx="1" fill="url(#clapGrad)" />
            <path d="M5.5 4L4 7.5M9.5 4L8 7.5M13.5 4L12 7.5" stroke="#ffffff" stroke-width="1.2" stroke-linecap="round" />
            <rect x="2.5" y="8" width="15" height="8" rx="1.2" fill="url(#clapGrad)" />
            <line x1="5" y1="11" x2="11" y2="11" stroke="#ffffff" stroke-opacity="0.8" stroke-width="1.2" stroke-linecap="round" />
            <line x1="5" y1="13.5" x2="9" y2="13.5" stroke="#ffffff" stroke-opacity="0.5" stroke-width="1.2" stroke-linecap="round" />
            <circle cx="14" cy="12.2" r="1.2" fill="#ffffff" fill-opacity="0.8" />
        </svg>
    }
}

/// The colour of a banner line's icon by what kind of line it is, so the pair is not always the same colour.
fn hook_color(kind: &str) -> &'static str {
    match kind {
        "new" => "text-danger-soft",
        "top10" => "text-orange-400",
        "credit" => "text-pink-300",
        "award" | "acclaimed" => "text-amber-300",
        "gem" => "text-cyan-300",
        "favourite" => "text-rose-400",
        "classic" => "text-yellow-200",
        "feel" => "text-amber-200",
        "mood" => "text-fuchsia-300",
        "late" => "text-indigo-300",
        "season" => "text-orange-300",
        "family" => "text-sky-300",
        "true_story" | "book" => "text-emerald-300",
        _ => "text-ink",
    }
}

/// The website's billboard (tablets and computers, note 114): the titles in turn, each with its trailer, logo, facts, lines and buttons.
#[component]
pub fn WebsiteHero(items: Vec<MediaItem>) -> impl IntoView {
    let ui_store = use_ui_store();
    let items_stored = StoredValue::new(items);
    let active_index = RwSignal::new(0usize);

    let (show_video, set_show_video) = signal(false);
    let (is_video_ready, set_is_video_ready) = signal(false);
    let (has_video_ended, set_has_video_ended) = signal(false);
    let (replay_count, set_replay_count) = signal(0u32);
    let (is_out_of_view, set_is_out_of_view) = signal(false);

    let show_timer = StoredValue::new(None::<i32>);
    let scroll_inactivity_timer = StoredValue::new(None::<i32>);
    let (is_unmounted_by_scroll, set_is_unmounted_by_scroll) = signal(false);

    // Get current item
    let current_item = move || {
        let idx = active_index.get_untracked();
        items_stored.with_value(|items| {
            items.get(idx).cloned().unwrap_or_default()
        })
    };

    let current_id = move || current_item().id;
    let current_is_tv = move || current_item().is_tv();
    let current_title = move || current_item().display_title().to_string();
    let current_backdrop = move || {
        let item = current_item();
        if let Some(bp) = item.backdrop_path.as_ref().or(item.poster_path.as_ref()) {
            if bp.starts_with("http") {
                bp.to_string()
            } else {
                format!("{}/original{}", crate::config::TMDB_IMAGE_URL, bp)
            }
        } else {
            String::new()
        }
    };
    let current_overview = move || current_item().overview;

    // Ambient background color synchronization with 0ms cache & fast w92 extraction
    Effect::new(move |_| {
        let bp = current_backdrop();
        if bp.is_empty() {
            return;
        }
        // 1. Synchronous cache check (0ms)
        if let Some(rgb) = crate::utils::ambient::get_cached_ambient_color(&bp) {
            ui_store.ambient_color.set(rgb);
            crate::utils::ambient::apply_ambient_color(rgb.0, rgb.1, rgb.2);
            return;
        }

        // 2. Ultra-fast thumbnail canvas sampling (~15ms)
        crate::utils::ambient::extract_ambient_color(&bp, move |maybe_rgb| {
            if let Some(rgb) = maybe_rgb {
                ui_store.ambient_color.set(rgb);
                crate::utils::ambient::apply_ambient_color(rgb.0, rgb.1, rgb.2);
            }
        });
    });

    // TMDB Logo resource
    let logo_resource = LocalResource::new(move || {
        let id = current_id();
        let tv = current_is_tv();
        async move {
            if id > 0 {
                fetch_movie_logo(id, tv).await.ok().flatten()
            } else {
                None
            }
        }
    });

    // TMDB Trailer video resource
    let videos_resource = LocalResource::new(move || {
        let id = current_id();
        let tv = current_is_tv();
        async move {
            // no trailer for the banner on a phone: it is not looked up, so no frame is made and nothing plays
            if id > 0 && !crate::form::current().is_phone() {
                fetch_videos(id, tv).await.unwrap_or_default()
            } else {
                vec![]
            }
        }
    });

    // chosen again when YouTube refuses the one playing (services/unplayable.rs): the next trailer plays in its place
    let best_trailer = move || {
        crate::services::unplayable::track();
        videos_resource.get().and_then(|videos| {
            crate::services::tmdb::select_best_tmdb_trailer(&videos)
        })
    };

    let trailer_key = move || {
        best_trailer().map(|(k, _)| k)
    };

    let is_teaser = move || {
        best_trailer().map(|(_, t)| t).unwrap_or(false)
    };

    // TMDB Details resource (genres, runtime/seasons, cast, release year)
    let details_resource = LocalResource::new(move || {
        let id = current_id();
        let tv = current_is_tv();
        async move {
            if id > 0 {
                fetch_details(id, tv).await.ok()
            } else {
                None
            }
        }
    });

    let media_type_label = move || {
        if current_is_tv() { crate::i18n::t("media-series") } else { crate::i18n::t("media-film") }
    };

    // The facts of the line under the title. A fact not known yet (details still loading, or not answered) is left out: the line used to say
    // "Action", "2024", "1 Season" or "1h 45m" instead, so Bones read "Action · 1 Season" until its details came.
    let genre_label = move || -> Option<String> {
        let from_details = details_resource.get().flatten().and_then(|d| d.genres.and_then(|g| g.first().map(|x| x.name.clone())));
        from_details
            .or_else(|| current_item().genre_ids.iter().find_map(|id| crate::data::page_genres::genre_name(*id)).map(str::to_string))
            .map(|name| crate::i18n::genre_label(&name))
    };

    let year_label = move || -> Option<String> {
        let year = |date: Option<&String>| date.filter(|d| d.len() >= 4).map(|d| d.chars().take(4).collect::<String>());
        if let Some(Some(ref d)) = details_resource.get() {
            if let Some(y) = year(d.release_date.as_ref().or(d.first_air_date.as_ref())) {
                return Some(y);
            }
        }
        let item = current_item();
        year(item.release_date.as_ref().or(item.first_air_date.as_ref()))
    };

    let duration_or_seasons_label = move || -> Option<String> {
        let details = details_resource.get().flatten()?;
        if current_is_tv() {
            let seasons = details.number_of_seasons.filter(|n| *n > 0)?;
            Some(crate::i18n::t_args("media-seasons", &[("n", fluent_bundle::FluentValue::from(seasons as i64))]))
        } else {
            let runtime = details.runtime.filter(|m| *m > 0)?;
            Some(if runtime >= 60 { format!("{}h {}m", runtime / 60, runtime % 60) } else { format!("{}m", runtime) })
        }
    };

    let meta_facts = move || -> Vec<String> {
        [Some(media_type_label()), genre_label(), year_label(), duration_or_seasons_label()].into_iter().flatten().collect()
    };

    // The lines the engine chose for this title (a mix of news, credits, fame, feel and season); the two old badges below are the fallback
    let engine_hooks = move || -> Vec<(String, String, String)> {
        current_item()
            .hooks
            .into_iter()
            .map(|h| {
                let text = crate::services::ai_engine::engine_text(&h.key, Some(&h.args)).unwrap_or(h.text.clone());
                (h.kind, h.icon, text)
            })
            .collect()
    };

    // Right badge, when the engine sent no lines: the director or creator from TMDB's credits
    let badge_right = move || {
        if let Some(Some(details)) = details_resource.get() {
            if let Some(credits) = details.credits {
                if let Some(director) = credits.crew.iter().find(|c| c.job.as_deref() == Some("Director") || c.job.as_deref() == Some("Creator")) {
                    let key = if current_is_tv() { "media-created-by" } else { "media-directed-by" };
                    return Some(("clapper".to_string(), crate::i18n::t_args(key, &[("name", fluent_bundle::FluentValue::from(director.name.clone()))])));
                }
            }
        }
        None
    };

    // Left badge, when the engine sent no lines: high acclaim or a well rated recent release
    let badge_left = move || {
        let yr = year_label().and_then(|y| y.parse::<u32>().ok()).unwrap_or(0);
        let vote = current_item().vote_average;
        if vote >= 8.2 {
            Some(("clapper".to_string(), crate::i18n::t("media-badge-masterpiece")))
        } else if vote >= 7.8 {
            Some(("clapper".to_string(), crate::i18n::t("media-badge-acclaimed")))
        } else if yr >= 2024 && vote >= 7.2 {
            Some(("megaphone".to_string(), crate::i18n::t("media-badge-trending-now")))
        } else {
            None
        }
    };

    // 2.5-second idle trailer playback trigger (Task 069)
    let start_trailer_timer = move || {
        clear_timeout_id(show_timer.get_value());
        set_show_video.set(false);
        set_is_video_ready.set(false);
        set_has_video_ended.set(false);

        let t = set_timeout_ms(move || {
            set_show_video.set(true);
        }, 2500);
        show_timer.set_value(t);
    };

    // Trigger trailer timer on mount or when active item changes
    Effect::new(move |_| {
        let _ = active_index.get();
        let _ = replay_count.get();
        start_trailer_timer();
    });

    // Scroll observer: immediate pause when scrolled out of view + 15-second RAM unmount
    let hero_container_ref = NodeRef::<leptos::html::Div>::new();
    crate::utils::listen::listen_window(leptos::ev::scroll, move |_| {
        if let Some(el) = hero_container_ref.get_untracked() {
            let rect = el.get_bounding_client_rect();
            let height = rect.height();
            if height > 0.0 {
                let visible_px = rect.bottom().min(height) - rect.top().max(0.0);
                let visible_fraction = (visible_px / height).max(0.0);
                let out = visible_fraction < 0.45;
                let was_out = is_out_of_view.get_untracked();

                if out != was_out {
                    set_is_out_of_view.set(out);

                    if out {
                        // Scrolled past hero: immediate pause
                        send_yt_command("hero-trailer-iframe", "pauseVideo");

                        // Start 15-second timer to unmount iframe & free system RAM / GPU
                        clear_timeout_id(scroll_inactivity_timer.get_value());
                        let t = set_timeout_ms(move || {
                            set_is_unmounted_by_scroll.set(true);
                            set_show_video.set(false);
                            set_is_video_ready.set(false);
                        }, 15_000);
                        scroll_inactivity_timer.set_value(t);
                    } else {
                        // Scrolled back into view
                        clear_timeout_id(scroll_inactivity_timer.get_value());
                        scroll_inactivity_timer.set_value(None);

                        if is_unmounted_by_scroll.get_untracked() {
                            // Was away > 15s: re-mount player cleanly
                            set_is_unmounted_by_scroll.set(false);
                            set_show_video.set(true);
                        } else {
                            // Was away < 15s: iframe was kept warm in DOM, resume immediately!
                            if show_video.get_untracked() && !has_video_ended.get_untracked() && !ui_store.info_modal_open.get_untracked() && ui_store.active_popup_id.get_untracked().is_none() {
                                send_yt_command("hero-trailer-iframe", "playVideo");
                            }
                        }
                    }
                }
            }
        }
    });

    on_cleanup(move || {
        clear_timeout_id(show_timer.get_value());
        clear_timeout_id(scroll_inactivity_timer.get_value());
    });

    fn send_yt_command(iframe_id: &str, func: &str) {
        if let Some(win) = web_sys::window() {
            if let Some(doc) = win.document() {
                if let Some(el) = doc.get_element_by_id(iframe_id) {
                    if let Ok(iframe) = el.dyn_into::<web_sys::HtmlIFrameElement>() {
                        if let Some(cw) = iframe.content_window() {
                            let cmd = format!(r#"{{"event":"command","func":"{}","args":[]}}"#, func);
                            let _ = cw.post_message(&wasm_bindgen::JsValue::from_str(&cmd), "*");
                        }
                    }
                }
            }
        }
    }


    let (hero_trailer_time, set_hero_trailer_time) = signal(0.0);

    // When video is verified playing, apply user mute preference if they had unmuted
    Effect::new(move |_| {
        if is_video_ready.get() && !ui_store.preview_muted.get() && !ui_store.info_modal_open.get() {
            send_yt_command("hero-trailer-iframe", "unMute");
        }
    });

    // Seamless Video Hand-Off: Pause on modal open, resume & sync timestamp on modal close
    Effect::new(move |_| {
        let modal_open = ui_store.info_modal_open.get();
        let paused_by_modal = ui_store.hero_paused_by_modal.get_untracked();

        if modal_open {
            // Modal just opened → immediately pause hero iframe
            send_yt_command("hero-trailer-iframe", "pauseVideo");
        } else if paused_by_modal {
            // Modal just closed → resume hero from exact timestamp
            ui_store.hero_paused_by_modal.set(false);
            let my_id = current_id();

            let mut resume_t: Option<f64> = None;

            // Use the timestamp the modal recorded when it closed
            if let Some((closed_id, final_t)) = ui_store.modal_closing_time.get_untracked() {
                ui_store.modal_closing_time.set(None);
                if closed_id == my_id && final_t > 0.0 {
                    set_hero_trailer_time.set(final_t);
                    set_has_video_ended.set(false);
                    set_show_video.set(true);
                    resume_t = Some(final_t);
                }
            }

            // Tell TrailerPlayer to seek to exact timestamp (updates internal anchor too)
            if let Some(t) = resume_t {
                ui_store.hero_resume_seek.set(Some(t));
            }

            // Resume playback smoothly in hero
            if show_video.get_untracked() && !has_video_ended.get_untracked() && !is_out_of_view.get_untracked() {
                send_yt_command("hero-trailer-iframe", "playVideo");
                let _ = set_timeout_ms(move || {
                    send_yt_command("hero-trailer-iframe", "playVideo");
                }, 80);
            }
        }
    });

    // Open modal handler with instant timestamp hand-off
    let trigger_open_modal = move || {
        let id = current_id();
        let tv = current_is_tv();
        let cur_t = hero_trailer_time.get();

        // Pause hero video while modal is active
        send_yt_command("hero-trailer-iframe", "pauseVideo");
        ui_store.hero_paused_by_modal.set(true);

        ui_store.modal_video_key.set(trailer_key());
        ui_store.modal_is_teaser.set(is_teaser());
        ui_store.modal_initial_time.set(cur_t);
        ui_store.modal_current_time.set(cur_t);
        ui_store.info_modal_movie_id.set(Some(id));
        ui_store.info_modal_is_tv.set(tv);
        ui_store.info_modal_open.set(true);
    };

    let should_play = move || {
        show_video.get()
            && trailer_key().is_some()
            && !has_video_ended.get()
            && !is_out_of_view.get()
            && !is_unmounted_by_scroll.get()
            && !ui_store.info_modal_open.get()
            && ui_store.active_popup_id.get().is_none()
    };

    let is_video_visible = move || should_play() && is_video_ready.get();

    // Three seconds after the trailer is on screen the description leaves (fading a little faster than it slides down), and the logo and the line under it slide down with it; the buttons stay.
    // It comes back when the trailer is not on screen any more (a new title, the end, the info window).
    let (synopsis_hidden, set_synopsis_hidden) = signal(false);
    Effect::new(move |_| {
        if is_video_visible() {
            let handle = set_timeout_with_handle(move || set_synopsis_hidden.set(true), std::time::Duration::from_millis(3000)).ok();
            on_cleanup(move || { if let Some(h) = handle { h.clear(); } });
        } else {
            set_synopsis_hidden.set(false);
        }
    });

    view! {
        <>
        // the billboard
        <div
            id="hero-container"
            node_ref=hero_container_ref
            class="w-full billboard-margin"
        >
            <div class="relative w-full aspect-[2.18/1] rounded-billboard overflow-hidden bg-raised border border-ink/[0.04] group">
                // ── Background Video Layer ──────────────────────────────────────
                <TrailerPlayer
                    video_key=Signal::derive(move || trailer_key())
                    is_teaser=Signal::derive(move || is_teaser())
                    is_playing=Signal::derive(move || should_play())
                    is_muted=ui_store.preview_muted
                    // NOTE: initial_seek_time is read ONCE at playback start (get_untracked inside TrailerPlayer).
                    // We pass hero_trailer_time as a frozen snapshot — TrailerPlayer only reads it untracked
                    // at the moment playback first starts, so this does NOT create a reactive loop.
                    // The `seek_time` prop handles all subsequent seeks (modal close hand-off).
                    initial_seek_time=Signal::derive(move || hero_trailer_time.get_untracked())
                    seek_time=Signal::derive(move || ui_store.hero_resume_seek.get())
                    variant="hero".to_string()
                    iframe_id="hero-trailer-iframe".to_string()
                    on_ready=Callback::new(move |_| {
                        set_is_video_ready.set(true);
                    })
                    on_ended=Callback::new(move |_| {
                        set_has_video_ended.set(true);
                    })
                    // a refused trailer is replaced by the next one (best_trailer), so the billboard does not count it as ended
                    on_unplayable=Callback::new(|_| {})
                    on_time_update=Callback::new(move |t| {
                        set_hero_trailer_time.set(t);
                        // Consume the one-shot seek signal once we confirm playback is updating
                        if ui_store.hero_resume_seek.get_untracked().is_some() {
                            ui_store.hero_resume_seek.set(None);
                        }
                    })
                />

                // ── Backdrop image with smooth cross-fade ────────────────────────
                <img
                    src=move || current_backdrop()
                    fetchpriority="high"
                    loading="eager"
                    alt=move || current_title()
                    class="absolute inset-0 w-full h-full object-cover object-[50%_20%] transition-opacity duration-(--motion-slowest) ease-in-out z-0"
                    class=("opacity-0", move || is_video_visible())
                    class=("opacity-100", move || !is_video_visible())
                />

                // ── Vignettes (Tuned down so backdrop image is clear & vibrant) ──
                <div
                    class="absolute inset-0 z-10 pointer-events-none bg-gradient-to-r from-void/70 via-void/25 to-transparent"
                />
                <div
                    class="absolute inset-x-0 bottom-0 h-44 z-10 pointer-events-none bg-gradient-to-t from-void/80 via-void/25 to-transparent"
                />
                <div
                    class="absolute inset-x-0 top-0 h-20 z-10 pointer-events-none bg-gradient-to-b from-void/35 to-transparent"
                />

                // ── Top-right reload/replay button ──────────────────────────────
                <MuteReplayButton
                    is_muted=ui_store.preview_muted
                    is_ended=Signal::derive(move || has_video_ended.get())
                    on_toggle=Callback::new(move |_| {
                        if has_video_ended.get() {
                            set_has_video_ended.set(false);
                            set_replay_count.update(|c| *c += 1);
                        } else {
                            let new_muted = !ui_store.preview_muted.get_untracked();
                            ui_store.set_preview_muted(new_muted);
                            send_yt_command("hero-trailer-iframe", if new_muted { "mute" } else { "unMute" });
                        }
                    })
                    on_force_replay=Callback::new(move |_| {
                        set_has_video_ended.set(false);
                        set_replay_count.update(|c| *c += 1);
                        send_yt_command("hero-trailer-iframe", "seekTo");
                        send_yt_command("hero-trailer-iframe", "playVideo");
                    })
                    size="sm".to_string()
                    class="absolute top-10 right-10 z-30 pointer-events-auto".to_string()
                />

                // ── Bottom Content Layer ─────────────────────────────────────────
                <div class="absolute inset-x-0 bottom-0 z-20 p-9 lg:p-10 flex flex-col md:flex-row md:items-end md:justify-between gap-6 pointer-events-none">

                    // Left Column: Logo/Title, Meta, Synopsis, CTA Buttons
                    <div class="flex flex-col items-start pointer-events-auto">

                        // Logo / Title with scale transition
                        <div
                            class="relative flex items-end origin-bottom-left"
                        >
                            <Suspense fallback=move || {
                                let t = current_title();
                                view! {
                                    <h1 class="text-4xl sm:text-6xl md:text-7xl lg:text-8xl font-black drop-shadow-title leading-none text-ink tracking-tight uppercase mb-1.5">
                                        {t}
                                    </h1>
                                }
                            }>
                                {move || {
                                    let t = current_title();
                                    logo_resource.get().map(|res| match res {
                                        Some(url) if !url.is_empty() => view! {
                                            <img
                                                src=url
                                                alt=t.clone()
                                                class="object-contain object-bottom drop-shadow-title mb-1.5"
                                                style="max-height: calc((100vw - 2 * var(--app-x)) * 0.12); max-width: calc((100vw - 2 * var(--app-x)) * 0.33);"
                                            />
                                        }.into_any(),
                                        _ => view! {
                                            <h1 class="text-4xl sm:text-6xl md:text-7xl lg:text-8xl font-black drop-shadow-title leading-none text-ink tracking-tight uppercase mb-1.5">
                                                {t}
                                            </h1>
                                        }.into_any(),
                                    })
                                }}
                            </Suspense>
                        </div>

                        // Metadata line: Series • Action • 2017 • 5 Seasons • [15]
                        <div class="flex items-center flex-wrap gap-2 mt-4 text-body-md text-ink font-medium select-none">
                            {move || meta_facts().into_iter().map(|fact| view! {
                                <span>{fact}</span>
                                <span class="text-ink/40 text-body-md">"•"</span>
                            }).collect_view()}
                            <div class="inline-flex items-center">
                                {move || {
                                    let item = current_item();
                                    view! {
                                        <RatingBadge id=item.id is_tv=item.is_tv() size="xs".to_string() />
                                    }
                                }}
                            </div>
                        </div>

                        // Overview / Synopsis
                        <div class=move || if synopsis_hidden.get() { "hero-synopsis is-hidden" } else { "hero-synopsis" }>
                            <div class="overflow-hidden min-h-0">
                                <p class="billboard-synopsis text-ink font-normal line-clamp-3 select-none">
                                    {move || current_overview()}
                                </p>
                            </div>
                        </div>

                        // Action Buttons: Play (pill) & More Info (pill)
                        <div class="flex items-center gap-3 mt-8 pointer-events-auto">
                            <PlayPillButton
                                href=Signal::derive(move || {
                                    let id = current_id();
                                    let is_tv = current_is_tv();
                                    let kind = if is_tv { "tv" } else { "movie" };
                                    let watch_store = crate::store::use_watch_store();
                                    if let Some(rec) = watch_store.get_record(id, is_tv) {
                                        if is_tv && rec.season.is_some() && rec.episode.is_some() {
                                            return format!("/watch/tv/{}?season={}&episode={}", id, rec.season.unwrap(), rec.episode.unwrap());
                                        }
                                    }
                                    format!("/watch/{}/{}", kind, id)
                                })
                                is_resume=Signal::derive(move || {
                                    let id = current_id();
                                    let is_tv = current_is_tv();
                                    let watch_store = crate::store::use_watch_store();
                                    if let Some(rec) = watch_store.get_record(id, is_tv) {
                                        if rec.percentage > 0.0 {
                                            return true;
                                        }
                                    }
                                    false
                                })
                                size="lg".to_string()
                            />

                            <MoreInfoPillButton
                                on_click=Callback::new(move |_| trigger_open_modal())
                                size="lg".to_string()
                            />
                        </div>
                    </div>

                    // Right Column: the lines of the banner, a different mix for each title
                    <div class="hidden lg:flex items-center gap-2 flex-shrink-0 pointer-events-auto select-none">
                        {move || {
                            let hooks = engine_hooks();
                            if !hooks.is_empty() {
                                return hooks.into_iter().map(|(kind, icon, text)| view! {
                                    <div class="flex items-center gap-1.5 text-label font-medium text-ink bg-void/50 p-2 rounded-card">
                                        <i class=format!("ph-bold ph-{} {}", icon, hook_color(&kind)) aria-hidden="true"></i>
                                        <span>{text}</span>
                                    </div>
                                }).collect::<Vec<_>>().into_any();
                            }
                            [badge_left(), badge_right()].into_iter().flatten().map(|(icon_type, text)| view! {
                                <div class="flex items-center gap-1.5 text-label font-medium text-ink bg-void/50 p-2 rounded-card">
                                    {match icon_type.as_str() {
                                        "megaphone" => view! { <MegaphoneBadgeIcon /> }.into_any(),
                                        _ => view! { <ClapperboardBadgeIcon /> }.into_any(),
                                    }}
                                    <span>{text}</span>
                                </div>
                            }).collect::<Vec<_>>().into_any()
                        }}
                    </div>
                </div>
            </div>
        </div>
        </>
    }
}
