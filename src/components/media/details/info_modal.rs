use leptos::prelude::*;
use leptos::html::Div;
use wasm_bindgen::prelude::*;
use crate::store::{use_ui_store, use_library_store, LibraryEntry};
use crate::services::tmdb::{fetch_details, fetch_movie_logo, fetch_videos, select_best_tmdb_trailer};
use crate::models::movie::Movie;
use crate::components::media::movie_card_badges::RatingBadge;
use crate::components::media::info_modal_episodes::InfoModalEpisodes;
use crate::components::media::info_modal_recommendations::InfoModalRecommendations;
use crate::components::media::action_buttons::{PlayPillButton, MyListButton, RatingPill, MuteReplayButton, CloseButton};
use crate::components::media::movie_card_rating::MovieRating;
use crate::components::trailer_player::TrailerPlayer;

fn send_yt_cmd(iframe_id: &str, func: &str) {
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

pub(crate) fn derive_vibe_tags(genres: &[(u32, String)], overview: &str) -> Vec<&'static str> {
    let lower_ov = overview.to_lowercase();
    let has_genre = |id: u32, name: &str| {
        genres.iter().any(|(gid, gn)| *gid == id || gn.to_lowercase().contains(name))
    };

    let mut tags = Vec::new();
    if has_genre(878, "sci-fi") || lower_ov.contains("quantum") || lower_ov.contains("timeline") || lower_ov.contains("simulation") {
        tags.push("Mind-Bending");
        tags.push("Cerebral");
    }
    if has_genre(27, "horror") || lower_ov.contains("curse") || lower_ov.contains("supernatural") || lower_ov.contains("haunting") {
        tags.push("Ominous");
        tags.push("Chilling");
    }
    if has_genre(53, "thriller") || has_genre(9648, "mystery") {
        if !tags.contains(&"Mind-Bending") {
            tags.push("Suspenseful");
        }
        tags.push("Psychological");
    }
    if has_genre(35, "comedy") {
        tags.push("Witty");
        tags.push("Irreverent");
    }
    if has_genre(80, "crime") {
        tags.push("Gritty");
        tags.push("Atmospheric");
    }
    if has_genre(28, "action") || has_genre(12, "adventure") {
        tags.push("High-Octane");
        tags.push("Exciting");
    }
    if has_genre(10749, "romance") {
        tags.push("Heartfelt");
        tags.push("Emotional");
    }
    if has_genre(16, "animation") || has_genre(10751, "family") {
        tags.push("Imaginative");
        tags.push("Charming");
    }
    if has_genre(99, "documentary") {
        tags.push("Provocative");
        tags.push("Eye-Opening");
    }
    if has_genre(18, "drama") && tags.is_empty() {
        tags.push("Compelling");
        tags.push("Emotional");
    }

    if tags.is_empty() {
        tags.push("Captivating");
        tags.push("Compelling");
    }

    tags.dedup();
    tags.truncate(3);
    tags
}

#[component]
pub fn InfoModal() -> impl IntoView {
    let ui_store = use_ui_store();
    let form = crate::form::use_form();
    let library_store = use_library_store();
    let is_open = ui_store.info_modal_open;
    let movie_id = ui_store.info_modal_movie_id;
    let is_tv = ui_store.info_modal_is_tv;

    let backdrop_ref = NodeRef::<Div>::new();
    let selected_season = RwSignal::new(1u32);

    // Video trailer state
    let (is_playing_trailer, set_is_playing_trailer) = signal(false);
    let (is_trailer_ready, set_is_trailer_ready) = signal(false);
    let is_muted = ui_store.preview_muted;
    let (video_key, set_video_key) = signal(None::<String>);
    let (is_teaser, set_is_teaser) = signal(false);
    let (has_ended, set_has_ended) = signal(false);

    let modal_trailer_banner_ref = NodeRef::<Div>::new();
    let (is_trailer_out_of_view, set_is_trailer_out_of_view) = signal(false);
    let modal_scroll_timer = StoredValue::new(None::<i32>);
    let (is_modal_unmounted_by_scroll, set_is_modal_unmounted_by_scroll) = signal(false);

    let close_cb = Callback::new(move |_| {
        let cur_t = ui_store.modal_current_time.get_untracked();
        let effective_t = if cur_t > 0.0 {
            cur_t
        } else {
            ui_store.modal_initial_time.get_untracked()
        };
        if let Some(id) = movie_id.get_untracked() {
            if effective_t > 0.0 {
                ui_store.modal_closing_time.set(Some((id, effective_t)));
            }
        }
        clear_timeout_id(modal_scroll_timer.get_value());
        modal_scroll_timer.set_value(None);
        set_is_trailer_out_of_view.set(false);
        set_is_modal_unmounted_by_scroll.set(false);
        ui_store.modal_video_key.set(None);
        ui_store.modal_initial_time.set(0.0);
        ui_store.modal_current_time.set(0.0);
        is_open.set(false);
        set_is_playing_trailer.set(false);
        set_is_trailer_ready.set(false);
        set_video_key.set(None);
        set_is_teaser.set(false);
        set_has_ended.set(false);
    });
    let on_close = move |_| close_cb.run(());

    let on_modal_scroll = move |_| {
        if let Some(el) = modal_trailer_banner_ref.get() {
            let rect = el.get_bounding_client_rect();
            let height = rect.height();
            if height > 0.0 {
                let visible_px = rect.bottom().min(height) - rect.top().max(0.0);
                let visible_fraction = (visible_px / height).max(0.0);
                let out = visible_fraction < 0.35 || rect.bottom() < 60.0;
                let was_out = is_trailer_out_of_view.get_untracked();

                if out != was_out {
                    set_is_trailer_out_of_view.set(out);

                    if out {
                        // Scrolled past modal trailer: immediate pause
                        send_yt_cmd("modal-trailer-iframe", "pauseVideo");

                        // Start 15-second timer to unmount trailer & free system RAM / GPU
                        clear_timeout_id(modal_scroll_timer.get_value());
                        let t = set_timeout_ms(move || {
                            set_is_modal_unmounted_by_scroll.set(true);
                            set_is_trailer_ready.set(false);
                        }, 15_000);
                        modal_scroll_timer.set_value(t);
                    } else {
                        // Scrolled back up to modal trailer
                        clear_timeout_id(modal_scroll_timer.get_value());
                        modal_scroll_timer.set_value(None);

                        if is_modal_unmounted_by_scroll.get_untracked() {
                            // Scrolled away > 15 seconds: re-mount trailer cleanly
                            set_is_modal_unmounted_by_scroll.set(false);
                        } else {
                            // Scrolled away < 15 seconds: warm in DOM, resume immediately!
                            if is_playing_trailer.get_untracked() && !has_ended.get_untracked() {
                                send_yt_cmd("modal-trailer-iframe", "playVideo");
                            }
                        }
                    }
                }
            }
        }
    };

    // Task 083: Escape key dismiss
    crate::utils::listen::listen_window(leptos::ev::keydown, move |ev: web_sys::KeyboardEvent| {
        if is_open.get() && ev.key() == "Escape" {
            close_cb.run(());
        }
    });

    // Task 084: Lock body scroll during modal presentation + scrollbar width compensation
    Effect::new(move |_| {
        let open = is_open.get();
        if let Some(w) = web_sys::window() {
            if let Some(doc) = w.document() {
                if let Some(body) = doc.body() {
                    if open {
                        let inner_w = w.inner_width().ok().and_then(|v| v.as_f64()).unwrap_or(0.0);
                        let client_w = doc.document_element().map(|el| el.client_width() as f64).unwrap_or(0.0);
                        let scrollbar_w = (inner_w - client_w).max(0.0);
                        let _ = body.style().set_property("overflow", "hidden");
                        if scrollbar_w > 0.0 {
                            let _ = body.style().set_property("padding-right", &format!("{}px", scrollbar_w));
                        }
                    } else {
                        let _ = body.style().set_property("overflow", "");
                        let _ = body.style().set_property("padding-right", "");
                    }
                }
            }
        }
    });

    on_cleanup(move || {
        if let Some(w) = web_sys::window() {
            if let Some(doc) = w.document() {
                if let Some(body) = doc.body() {
                    let _ = body.style().set_property("overflow", "");
                    let _ = body.style().set_property("padding-right", "");
                }
            }
        }
    });

    // Reset state when movie_id or is_tv changes
    Effect::new(move |_| {
        let _ = movie_id.get();
        let _ = is_tv.get();
        selected_season.set(1);
        set_is_playing_trailer.set(false);
        set_is_trailer_ready.set(false);
        set_video_key.set(None);
        set_is_teaser.set(false);
        set_has_ended.set(false);
    });

    let details_resource = LocalResource::new(move || {
        let id_opt = movie_id.get();
        let is_tv_val = is_tv.get();
        async move {
            if let Some(id) = id_opt {
                fetch_details(id, is_tv_val).await.inspect_err(|e| leptos::logging::error!("Fetch details failed: {:?}", e)).ok()
            } else {
                None
            }
        }
    });

    let logo_resource = LocalResource::new(move || {
        let id_opt = movie_id.get();
        let is_tv_val = is_tv.get();
        async move {
            if let Some(id) = id_opt {
                fetch_movie_logo(id, is_tv_val).await.ok().flatten()
            } else {
                None
            }
        }
    });

    // Trailer fetch and auto-preview
    Effect::new(move |_| {
        let id_opt = movie_id.get();
        let is_tv_val = is_tv.get();
        let open = is_open.get();

        if open {
            if let Some(id) = id_opt {
                if let Some(key) = ui_store.modal_video_key.get_untracked() {
                    set_video_key.set(Some(key));
                    set_is_teaser.set(ui_store.modal_is_teaser.get_untracked());
                    set_is_playing_trailer.set(true);
                } else {
                    leptos::task::spawn_local(async move {
                        if let Ok(videos) = fetch_videos(id, is_tv_val).await {
                            if let Some((best_key, teaser_flag)) = select_best_tmdb_trailer(&videos) {
                                set_video_key.set(Some(best_key));
                                set_is_teaser.set(teaser_flag);
                                // Auto-play immediately (0ms delay) if handed off with an active timestamp,
                                // or after a brief 500ms transition delay for fresh browses.
                                let init_t = ui_store.modal_initial_time.get_untracked();
                                let delay_ms = if init_t > 0.0 { 0 } else { 500 };
                                let _ = leptos::leptos_dom::helpers::set_timeout_with_handle(
                                    move || {
                                        set_is_playing_trailer.set(true);
                                    },
                                    std::time::Duration::from_millis(delay_ms),
                                );
                            }
                        }
                    });
                }
            }
        }
    });

    let on_recommendation_click = Callback::new(move |rec: Movie| {
        let rec_id = rec.id_u32();
        let rec_is_tv = rec.is_tv();
        movie_id.set(Some(rec_id));
        is_tv.set(rec_is_tv);
        selected_season.set(1);
        set_is_playing_trailer.set(false);
        set_is_trailer_ready.set(false);
        set_video_key.set(None);
        set_is_teaser.set(false);
        set_has_ended.set(false);
        ui_store.modal_video_key.set(None);
        ui_store.modal_initial_time.set(0.0);
        ui_store.modal_current_time.set(0.0);

        if let Some(backdrop_el) = backdrop_ref.get() {
            backdrop_el.set_scroll_top(0);
        }
    });

    view! {
        // Modal Backdrop (Task 083: backdrop click dismiss)
        <div
            node_ref=backdrop_ref
            class="fixed inset-0 z-(--z-info-modal) bg-void/70 flex justify-center overflow-y-auto scrollbar-hide cursor-default transition-opacity duration-(--motion-slow)"
            class=("opacity-100", move || is_open.get())
            class=("pointer-events-auto", move || is_open.get())
            class=("opacity-0", move || !is_open.get())
            class=("pointer-events-none", move || !is_open.get())
            on:click=on_close
            on:scroll=on_modal_scroll
        >
            // Modal Container
            <div
                class="relative w-full max-w-[850px] bg-raised rounded-sheet mt-6 md:mt-8 mb-8 overflow-hidden h-fit mx-4 transition-transform duration-(--motion-slow) origin-center"
                class=("scale-100", move || is_open.get())
                class=("scale-95", move || !is_open.get())
                on:click=|e| e.stop_propagation()
            >
                // Floating Close Button
                <CloseButton
                    on_close=close_cb
                    size="md".to_string()
                    class="absolute top-4 right-4 z-50".to_string()
                />

                <Suspense fallback=move || view! { <crate::components::skeleton::DetailsSkeleton /> }>
                    {move || details_resource.get().map(|res| match res {
                        Some(details) => {
                            let title = details.display_title().to_string();
                            let fallback_title = title.clone();
                            let backdrop = details.backdrop_path.as_ref().map(|p| format!("{}/original{}", crate::config::TMDB_IMAGE_URL, p)).unwrap_or_default();
                            let overview = details.overview;
                            let release_year = details.release_date.or(details.first_air_date)
                                .map(|d| d.chars().take(4).collect::<String>()).unwrap_or_default();
                            let runtime = details.runtime.map(|r| format!("{}h {}m", r / 60, r % 60));
                            let seasons_count = details.number_of_seasons.unwrap_or(0);
                            let seasons_label = if seasons_count > 0 {
                                Some(crate::i18n::t_args("media-seasons", &[("n", fluent_bundle::FluentValue::from(seasons_count as i64))]))
                            } else {
                                None
                            };
                            let duration = runtime.or(seasons_label).unwrap_or_default();
                            let cast_items = details.credits.map(|c| c.cast.into_iter().take(4).map(|cm| cm.name).collect::<Vec<_>>()).unwrap_or_default();
                            let genres_items = details.genres.clone().map(|g| g.into_iter().map(|gx| (gx.id, gx.name)).collect::<Vec<_>>()).unwrap_or_default();
                            let match_norm = ((details.vote_average - 5.5) / 3.3).clamp(0.0, 1.0);
                            let match_pct = (75.0 + match_norm * 23.0).round() as u32;
                            let match_score = crate::i18n::t_args("media-percent-match", &[("pct", fluent_bundle::FluentValue::from(match_pct as i64))]);
                            let vibe_tags = derive_vibe_tags(&genres_items, &overview);
                            let vibe_text = vibe_tags.iter().map(|t| crate::i18n::mood_label(t)).collect::<Vec<_>>().join(", ");

                            let current_movie_id = details.id;
                            let is_tv_val = details.number_of_seasons.is_some();
                            let is_in_list = Signal::derive(move || {
                                library_store.my_list.get().contains_key(&current_movie_id)
                            });

                            let toggle_my_list_cb = {
                                let title_c = title.clone();
                                let backdrop_c = details.backdrop_path.clone();
                                let overview_c = overview.clone();
                                Callback::new(move |_| {
                                    let mut list = library_store.my_list.get();
                                    if list.contains_key(&current_movie_id) {
                                        list.remove(&current_movie_id);
                                    } else {
                                        list.insert(current_movie_id, LibraryEntry {
                                            media: crate::services::tmdb::MediaItem {
                                                id: current_movie_id,
                                                title: Some(title_c.clone()),
                                                name: Some(title_c.clone()),
                                                overview: overview_c.clone(),
                                                poster_path: backdrop_c.clone(),
                                                backdrop_path: backdrop_c.clone(),
                                                vote_average: details.vote_average,
                                                release_date: None,
                                                first_air_date: None,
                                                media_type: if is_tv_val { Some("tv".to_string()) } else { Some("movie".to_string()) },
                                                ..Default::default()
                                            },
                                            added_at: js_sys::Date::now() as u64,
                                        });
                                    }
                                    library_store.my_list.set(list);
                                })
                            };

                            let (modal_rating, set_modal_rating) = signal(crate::store::ratings::get(current_movie_id, is_tv_val));

                            // Share opens the device's share sheet with a link that opens this title (see SharedTitleOpener)
                            let share_open = RwSignal::new(false);
                            let share_url = crate::pages::title::share_url(is_tv_val, current_movie_id);
                            let share_title = title.clone();

                            let watch_store = crate::store::use_watch_store();
                            let watch_record = watch_store.get_record(current_movie_id, is_tv_val);
                            let has_progress = watch_record.as_ref().map(|r| r.percentage > 0.0).unwrap_or(false);

                            let watch_url = if is_tv_val {
                                if let Some(ref rec) = watch_record {
                                    if let (Some(s), Some(e)) = (rec.season, rec.episode) {
                                        format!("/watch/tv/{}?season={}&episode={}", current_movie_id, s, e)
                                    } else {
                                        format!("/watch/tv/{}?season={}&episode=1", current_movie_id, selected_season.get())
                                    }
                                } else {
                                    format!("/watch/tv/{}?season={}&episode=1", current_movie_id, selected_season.get())
                                }
                            } else {
                                format!("/watch/movie/{}", current_movie_id)
                            };

                            view! {
                                <>
                                    // Hero Banner Container
                                    <div
                                        node_ref=modal_trailer_banner_ref
                                        class="relative aspect-video w-full bg-void group overflow-hidden"
                                    >
                                        // Static Backdrop Image, in the size this screen needs (utils/images.rs)
                                        <img
                                            srcset=crate::utils::images::backdrop_srcset(&backdrop)
                                            sizes="(min-width: 850px) 850px, 100vw"
                                            src=backdrop
                                            class="w-full h-full object-cover scale-[1.05] transition-opacity duration-(--motion-slowest)"
                                            class=("opacity-0", move || is_trailer_ready.get() && is_playing_trailer.get() && !has_ended.get() && !is_trailer_out_of_view.get())
                                            class=("opacity-100", move || !is_trailer_ready.get() || !is_playing_trailer.get() || has_ended.get() || is_trailer_out_of_view.get())
                                            alt=fallback_title.clone()
                                        />

                                        // Video Trailer Layer (TrailerPlayer)
                                        <TrailerPlayer
                                            video_key=Signal::derive(move || video_key.get())
                                            is_teaser=Signal::derive(move || is_teaser.get())
                                            is_playing=Signal::derive(move || is_playing_trailer.get() && !has_ended.get() && !is_trailer_out_of_view.get() && !is_modal_unmounted_by_scroll.get())
                                            is_muted=ui_store.preview_muted
                                            initial_seek_time=Signal::derive(move || ui_store.modal_initial_time.get())
                                            variant="modal".to_string()
                                            iframe_id="modal-trailer-iframe".to_string()
                                            on_ready=Callback::new(move |_| {
                                                set_is_trailer_ready.set(true);
                                                // the title goes to "Trailers You've Watched"
                                                if let Some(id) = movie_id.get_untracked() { crate::store::trailers::mark(id, is_tv.get_untracked()); }
                                            })
                                            on_ended=Callback::new(move |_| {
                                                set_has_ended.set(true);
                                                set_is_playing_trailer.set(false);
                                                set_is_trailer_ready.set(false);
                                            })
                                            // a trailer YouTube refuses here gives way to the title's next one (if the window still shows that title),
                                            // or ends as above
                                            on_unplayable=Callback::new(move |_| {
                                                set_is_trailer_ready.set(false);
                                                let (Some(id), tv) = (movie_id.get_untracked(), is_tv.get_untracked()) else { return };
                                                leptos::task::spawn_local(async move {
                                                    let next = crate::services::unplayable::next_trailer(id, tv).await;
                                                    if movie_id.try_get_untracked() != Some(Some(id)) {
                                                        return;
                                                    }
                                                    match next {
                                                        Some((key, teaser)) => {
                                                            set_is_teaser.try_set(teaser);
                                                            set_video_key.try_set(Some(key));
                                                        }
                                                        None => {
                                                            set_has_ended.try_set(true);
                                                            set_is_playing_trailer.try_set(false);
                                                        }
                                                    }
                                                });
                                            })
                                            on_time_update=Callback::new(move |t| {
                                                ui_store.modal_current_time.set(t);
                                            })
                                        />

                                        // Cinematic Gradient Overlay (bottom 40% blend)
                                        <div class="absolute inset-x-0 bottom-0 h-1/2 bg-gradient-to-t from-raised to-transparent z-10 pointer-events-none" />

                                        // Title / Logo & Action Buttons Overlay: on a computer 13 % of the trailer's height above its bottom, as Netflix's (measured 2026-10-23 at
                                        // 1736 wide: Play centred 86.5 px above the trailer's bottom edge, 481 tall)
                                        <div class="absolute bottom-6 left-6 md:bottom-[13%] md:left-12 w-[340px] max-w-[70%] z-20 pointer-events-auto">
                                            // Logo or Title
                                            <div class="relative flex items-end mb-6">
                                                <Suspense fallback=move || view! { <h2 class="text-ink text-3xl sm:text-4xl font-bold mb-4">{fallback_title.clone()}</h2> }>
                                                    {
                                                        let tc = title.clone();
                                                        move || {
                                                            let t = tc.clone();
                                                            logo_resource.get().map(|res| match res {
                                                                Some(url) if !url.is_empty() => view! {
                                                                    <img
                                                                        src=url.clone()
                                                                        alt=t.clone()
                                                                        class="object-contain object-bottom drop-shadow-xl"
                                                                        style="max-height: 136px; max-width: 340px;"
                                                                    />
                                                                }.into_any(),
                                                                _ => view! { <h2 class="text-ink text-3xl sm:text-4xl md:text-5xl font-black font-brand drop-shadow-xl leading-none tracking-wide">{t.clone()}</h2> }.into_any()
                                                            })
                                                        }
                                                    }
                                                </Suspense>
                                            </div>

                                            // How far the viewer is (a started title): the bar and `16 of 43m`, above the buttons, as in the app
                                            {watch_record.as_ref().filter(|_| has_progress).map(|rec| {
                                                let pct = rec.percentage.clamp(2.0, 100.0);
                                                let time_text = crate::i18n::t_args("continue-watching-time-of", &[
                                                    ("done", fluent_bundle::FluentValue::from((rec.current_time / 60.0).round() as i64)),
                                                    ("total", fluent_bundle::FluentValue::from((rec.duration / 60.0).round() as i64)),
                                                ]);
                                                view! {
                                                    <div class="mb-4 flex items-center gap-3">
                                                        <div class="h-[3px] w-[310px] max-w-[70%] overflow-hidden rounded-full bg-ink/40">
                                                            <div class="h-full bg-brand" style=format!("width: {:.1}%;", pct) />
                                                        </div>
                                                        <span class="text-ink-silver text-body-md tabular-nums">{time_text}</span>
                                                    </div>
                                                }
                                            })}

                                            // CTA Action Buttons
                                            <div class="flex items-center flex-wrap gap-2 sm:gap-3">
                                                <PlayPillButton
                                                    href=watch_url
                                                    is_resume=has_progress
                                                    size="lg".to_string()
                                                    shape="pill".to_string()
                                                    class="scale-[0.99]"
                                                />

                                                // A started title: mark as watched (takes it out of Continue Watching); otherwise the My List button
                                                {if has_progress {
                                                    view! {
                                                        <crate::components::media::action_buttons::CircleIconButton
                                                            icon="ph-check"
                                                            on_click=Callback::new(move |_| crate::store::use_watch_store().remove_record(current_movie_id, is_tv_val))
                                                            title=crate::i18n::t("continue-watching-mark-watched")
                                                        />
                                                    }.into_any()
                                                } else {
                                                    view! {
                                                        <MyListButton
                                                            is_in_list=is_in_list
                                                            on_toggle=toggle_my_list_cb
                                                            size="md".to_string()
                                                            show_tooltip=true
                                                        />
                                                    }.into_any()
                                                }}

                                                // Thumbs Up / Rate Button
                                                <RatingPill
                                                    rating=modal_rating
                                                    on_set=Callback::new(move |r: Option<MovieRating>| {
                                                        set_modal_rating.set(r);
                                                        crate::store::ratings::set(current_movie_id, is_tv_val, r);
                                                    })
                                                    size="md".to_string()
                                                />

                                                // Share: the phone form only, as in Netflix's apps (its website has none)
                                                <Show when=move || form.get().is_phone()>
                                                    <crate::components::media::action_buttons::CircleIconButton
                                                        icon="ph-share-network"
                                                        on_click=Callback::new(move |_| share_open.set(true))
                                                        title=crate::i18n::t("share-button")
                                                    />
                                                </Show>
                                            </div>
                                            <crate::components::media::share_sheet::ShareSheet open=share_open title=share_title url=share_url />
                                        </div>

                                        // Mute / Replay Button
                                        {move || if video_key.get().is_some() {
                                            view! {
                                                <MuteReplayButton
                                                    is_muted=is_muted
                                                    is_ended=has_ended
                                                    on_toggle=Callback::new(move |_| {
                                                        if has_ended.get() {
                                                            set_has_ended.set(false);
                                                            set_is_playing_trailer.set(true);
                                                        } else {
                                                            let new_muted = !ui_store.preview_muted.get_untracked();
                                                            ui_store.set_preview_muted(new_muted);
                                                        }
                                                    })
                                                    size="md".to_string()
                                                    // on the Play row's centre line and at the content's right edge (48 px), as Netflix's
                                                    class="absolute bottom-6 right-6 md:bottom-[calc(13%+4px)] md:right-12 z-30 pointer-events-auto".to_string()
                                                />
                                            }.into_any()
                                        } else {
                                            view! { <span /> }.into_any()
                                        }}
                                    </div>

                                    // Content Details Section
                                    <div class="px-6 md:px-12 pb-8 bg-raised">
                                        <div class="grid grid-cols-1 md:grid-cols-[2fr_1fr] gap-x-8 gap-y-5 pt-3">
                                            // Left Column: Match, Year, HD, Duration, Maturity, Overview
                                            <div class="space-y-4">
                                                <div class="flex flex-wrap items-center gap-x-3 gap-y-2 text-ink font-normal text-base font-brand">
                                                    <span class="text-success font-extrabold tracking-wide">{match_score}</span>
                                                    <span class="text-ink-silver">{release_year}</span>
                                                    <RatingBadge id=current_movie_id is_tv=is_tv_val size="xs".to_string() />
                                                    <span class="border border-ink/40 px-[5.6px] text-ink/90 text-caption-sm flex items-center rounded-tick">"HD"</span>
                                                    <span class="text-ink-silver">{duration}</span>
                                                </div>

                                                {watch_record.as_ref().filter(|r| has_progress && r.is_tv).and_then(|r| match (r.season, r.episode) {
                                                    (Some(se), Some(ep)) => Some(match r.episode_title.as_deref().filter(|t| !t.is_empty()) {
                                                        Some(t) => format!("S{}:E{} \"{}\"", se, ep, t),
                                                        None => format!("S{}:E{}", se, ep),
                                                    }),
                                                    _ => None,
                                                }).map(|line| view! { <p class="text-ink font-bold text-title-md leading-tight">{line}</p> })}

                                                // Netflix's synopsis here: 16 px on 26 px lines
                                                <p class="text-ink font-normal text-body-md leading-[26px] pt-1">
                                                    {overview}
                                                </p>
                                            </div>

                                            // Right Column: Cast, Genres, Tags
                                            <div class="space-y-4 pt-1 text-sm">
                                                {if !cast_items.is_empty() {
                                                    let cast_rendered = cast_items.iter().map(|actor| {
                                                        let a = actor.clone();
                                                            let query_str = String::from(js_sys::encode_uri_component(&a));
                                                            view! {
                                                                <a
                                                                    href=format!("/search?q={}", query_str)
                                                                    class="text-ink font-normal hover:underline cursor-pointer"
                                                                >
                                                                    {a}
                                                                </a>
                                                            }
                                                    }).collect_view();

                                                    view! {
                                                        <div class="flex flex-wrap gap-x-1">
                                                            <span class="text-ink-ash font-normal mr-1">{crate::i18n::t("media-cast")}</span>
                                                            {cast_rendered}
                                                        </div>
                                                    }.into_any()
                                                } else {
                                                    view! { <span /> }.into_any()
                                                }}

                                                {if !genres_items.is_empty() {
                                                    let genres_rendered = genres_items.iter().map(|(gid, gname)| {
                                                        let gn = gname.clone();
                                                        let target_route = format!("/browse/genre-{}", gid);
                                                        view! {
                                                            <a
                                                                href=target_route
                                                                class="text-ink font-normal hover:underline cursor-pointer"
                                                            >
                                                                {gn}
                                                            </a>
                                                        }
                                                    }).collect_view();

                                                    view! {
                                                        <div class="flex flex-wrap gap-x-1">
                                                            <span class="text-ink-ash font-normal mr-1">{crate::i18n::t("media-genres")}</span>
                                                            {genres_rendered}
                                                        </div>
                                                    }.into_any()
                                                } else {
                                                    view! { <span /> }.into_any()
                                                }}

                                                {
                                                    let vibe_label = if is_tv_val { crate::i18n::t("media-this-show-is") } else { crate::i18n::t("media-this-movie-is") };
                                                    view! {
                                                        <div class="flex flex-wrap gap-x-1">
                                                            <span class="text-ink-ash font-normal mr-1">{vibe_label}</span>
                                                            <span class="text-ink font-normal">{vibe_text.clone()}</span>
                                                        </div>
                                                    }
                                                }
                                            </div>
                                        </div>

                                        // TV Shows: Episodes Selector & Episode List
                                        {if is_tv_val && seasons_count > 0 {
                                            view! {
                                                <div class="mt-4">
                                                    <InfoModalEpisodes
                                                        series_id=current_movie_id
                                                        total_seasons=seasons_count
                                                        selected_season=selected_season
                                                    />
                                                </div>
                                            }.into_any()
                                        } else {
                                            view! { <span /> }.into_any()
                                        }}

                                        // Recommendations: More Like This Grid
                                        <div>
                                            <InfoModalRecommendations
                                                movie_id=current_movie_id
                                                is_tv=is_tv_val
                                                on_recommendation_click=on_recommendation_click
                                            />
                                        </div>
                                    </div>
                                </>
                            }.into_any()
                        },
                        // the message only exists while the window is open, so it is not read out or indexed on every page
                        None if !is_open.get() => view! { <></> }.into_any(),
                        None => view! {
                            <div class="w-full h-40 text-ink p-10 flex items-center justify-center">
                                {crate::i18n::t("info-modal-error-loading-details")}
                            </div>
                        }.into_any()
                    })}
                </Suspense>
            </div>
        </div>
    }
}

/// `/browse?title=tv-1399` opens that title's details once a profile is chosen. A shared link (`/title/tv-1399`) comes here when the
/// browser holds an account, and so does signing in from a title's page. Without an account it opens only when the engine says accounts
/// are optional (the gate's rule); otherwise the gate sends it to sign in, which keeps the whole address in `next`. The parameter is then
/// taken out of the address so a reload does not open it again.
#[component]
pub fn SharedTitleOpener() -> impl IntoView {
    let location = leptos_router::hooks::use_location();
    let ui_store = use_ui_store();
    let profile_store = crate::store::use_profile_store();
    let handled = StoredValue::new(String::new());
    let open = move |id: u32, is_tv: bool| {
        ui_store.modal_initial_time.set(0.0);
        ui_store.modal_current_time.set(0.0);
        ui_store.info_modal_movie_id.set(Some(id));
        ui_store.info_modal_is_tv.set(is_tv);
        ui_store.info_modal_open.set(true);
        if let Some(h) = web_sys::window().and_then(|w| w.history().ok()) {
            let _ = h.replace_state_with_url(&JsValue::NULL, "", Some("/browse"));
        }
    };
    Effect::new(move |_| {
        let path = location.pathname.get();
        let search = location.search.get();
        if path != "/browse" || profile_store.active_profile_id.get().is_none() || handled.with_value(|h| *h == search) {
            return;
        }
        let Some((kind, id)) = search.trim_start_matches('?').split('&').find_map(|kv| kv.strip_prefix("title=")).and_then(|v| v.split_once('-')) else { return };
        let (Ok(id), true) = (id.parse::<u32>(), kind == "tv" || kind == "movie") else { return };
        let is_tv = kind == "tv";
        handled.set_value(search.clone());
        if crate::services::account::has_account() {
            open(id, is_tv);
        } else {
            leptos::task::spawn_local(async move {
                if crate::services::ai_engine::fetch_features().await.get("accounts_required") == Some(&false) {
                    open(id, is_tv);
                }
            });
        }
    });
}
