use leptos::prelude::*;
use leptos::portal::Portal;
use wasm_bindgen::prelude::*;
use crate::models::movie::Movie;
use crate::components::media::movie_card_badges::RatingBadge;
use crate::components::media::movie_card_rating::MovieRating;
use crate::components::media::action_buttons::{
    PlayCircleButton, MyListButton, RatingPill, MoreInfoCaretButton, MuteReplayButton,
};
use crate::components::trailer_player::TrailerPlayer;

/// Netflix preview card width is 1.5 x the tile width (435 for a 290 wide tile). Narrow ranked posters keep 341.
pub fn popup_w(tile_w: f64) -> f64 {
    (tile_w * 1.5).round().max(341.0)
}

/// Card height used for placement: 16:9 media plus the 167px info panel (413 for a 435 wide card).
fn popup_h(w: f64) -> f64 {
    w * 9.0 / 16.0 + 167.0
}

pub fn calc_popup_position(
    rect: &web_sys::DomRect,
    pos: &str,
    initial_scroll: (f64, f64),
    window_w: f64,
    _window_h: f64,
) -> (f64, f64) {
    let app_x = crate::utils::layout::app_x();
    let popup_width = popup_w(rect.width());

    let mut left = if pos == "left" {
        app_x
    } else if pos == "right" {
        window_w - app_x - popup_width
    } else {
        rect.left() + rect.width() / 2.0 - popup_width / 2.0
    };

    left = left.max(app_x).min(window_w - popup_width - app_x);

    let (scroll_x, scroll_y) = initial_scroll;
    // Netflix centres the card vertically on the tile (card centre y equals tile centre y, measured at 1578 wide).
    let centre = rect.top() + rect.height() / 2.0;
    let page_top = (scroll_y + centre - popup_h(popup_width) / 2.0).max(scroll_y + 80.0);
    let page_left = scroll_x + left;

    (page_top, page_left)
}

pub fn derive_vibe_tags(
    vibe_pills: Option<&[String]>,
    genre_ids: Option<&[i64]>,
    overview: &str,
) -> Vec<String> {
    let mut tags: Vec<String> = Vec::new();

    // 1. Existing vibe pills if present
    if let Some(pills) = vibe_pills {
        for pill in pills {
            let p = pill.trim().to_string();
            if !p.is_empty() && !tags.iter().any(|t| t.eq_ignore_ascii_case(&p)) {
                tags.push(p);
                if tags.len() >= 3 {
                    return tags;
                }
            }
        }
    }

    let ov_lower = overview.to_lowercase();

    // 2. Thematic cues from overview text
    let thematic_cues = [
        (&["silo", "dystop", "post-apoc", "apocalyp", "bunker", "toxic future", "ruined future"][..], "Dystopian"),
        (&["cyberpunk", "artificial intelligence", "cyborg", "robot", "virtual reality", "simulation"][..], "Cyberpunk"),
        (&["space", "galaxy", "alien", "interstellar", "starship", "planet", "extraterrestrial"][..], "Mind-Bending"),
        (&["time travel", "multiverse", "timeline", "paradox"][..], "Mind-Bending"),
        (&["conspiracy", "secrets", "lies", "deception", "whistleblower", "cover-up"][..], "Suspenseful"),
        (&["detective", "murder", "investigat", "serial killer", "crime syndicate", "underworld"][..], "Gritty"),
        (&["psychological", "paranoia", "sanity", "hallucination", "nightmare"][..], "Psychological"),
        (&["survival", "survive", "stranded", "hostile", "deadly"][..], "Edge-of-Your-Seat"),
        (&["heist", "thief", "steal", "con artist", "casino"][..], "Slick"),
        (&["supernatural", "ghost", "haunt", "demon", "curse", "witch"][..], "Ominous"),
        (&["romantic", "falls in love", "heartbreak", "wedding", "passion"][..], "Heartfelt"),
        (&["satire", "parody", "hilarious", "misadventures", "absurd"][..], "Irreverent"),
        (&["coming of age", "adolescen", "high school", "childhood", "growing up"][..], "Coming-of-Age"),
        (&["epic battle", "empire", "kingdom", "dynasty", "warrior", "rebellion"][..], "Epic"),
    ];

    for (keywords, label) in thematic_cues {
        if keywords.iter().any(|&kw| ov_lower.contains(kw)) {
            if !tags.iter().any(|t| t.eq_ignore_ascii_case(label)) {
                tags.push(label.to_string());
                if tags.len() >= 3 {
                    return tags;
                }
            }
        }
    }

    // 3. Genre ID mapping
    if let Some(genres) = genre_ids {
        for &gid in genres {
            let genre_tags: &[&str] = match gid {
                28 => &["High-Octane", "Action"],
                12 => &["Adventurous", "Thrilling"],
                16 => &["Imaginative", "Charming"],
                35 => &["Witty", "Feel-Good"],
                80 => &["Gritty", "Dark"],
                99 => &["Provocative", "Insightful"],
                18 => &["Compelling", "Emotional"],
                10751 => &["Heartwarming", "Family"],
                14 => &["Enchanting", "Mythical"],
                36 => &["Sweeping", "Historical"],
                27 => &["Chilling", "Terrifying"],
                10402 => &["Soulful", "Rhythmic"],
                9648 => &["Mysterious", "Intriguing"],
                10749 => &["Romantic", "Passionate"],
                878 => &["Sci-Fi", "Futuristic"],
                53 => &["Suspenseful", "Gripping"],
                10752 => &["Visceral", "Intense"],
                37 => &["Atmospheric", "Raw"],
                10759 => &["Thrilling", "Action"],
                10762 => &["Playful", "Wholesome"],
                10764 => &["Unscripted", "Dramatic"],
                10765 => &["Sci-Fi", "Imaginative"],
                10768 => &["Political", "Intense"],
                _ => &[],
            };

            for &gt in genre_tags {
                if !tags.iter().any(|t| t.eq_ignore_ascii_case(gt)) {
                    tags.push(gt.to_string());
                    if tags.len() >= 3 {
                        return tags;
                    }
                }
            }
        }
    }

    // 4. Fallback pool if fewer than 3
    let fallback_pool = ["Compelling", "Suspenseful", "Exciting", "Atmospheric", "Intriguing"];
    for &fb in &fallback_pool {
        if !tags.iter().any(|t| t.eq_ignore_ascii_case(fb)) {
            tags.push(fb.to_string());
            if tags.len() >= 3 {
                break;
            }
        }
    }

    tags.truncate(3);
    tags
}

/// What the hover card of a Continue Watching tile shows: the episode, how far the viewer is, and the buttons that take the title out of the row.
#[derive(Clone)]
pub struct ResumeInfo {
    /// `S1:E22 "Blood Brothers"`, or the title of a film
    pub label: String,
    /// `16 of 43m`
    pub time_text: String,
    pub percent: f32,
    pub on_remove: Callback<()>,
}

#[component]
pub fn MovieCardPopup(
    movie: Movie,
    hovered_rect: web_sys::DomRect,
    initial_scroll: (f64, f64),
    #[prop(default = "center".to_string())] hover_position: String,
    on_mouse_enter: Callback<()>,
    on_mouse_leave: Callback<()>,
    on_play: Callback<()>,
    on_open_modal: Callback<()>,
    #[prop(default = None)] resume: Option<ResumeInfo>,
) -> impl IntoView {
    let (window_w, window_h) = web_sys::window()
        .map(|w| {
            let w_val = w.inner_width().ok().and_then(|v| v.as_f64()).unwrap_or(1200.0);
            let h_val = w.inner_height().ok().and_then(|v| v.as_f64()).unwrap_or(800.0);
            (w_val, h_val)
        })
        .unwrap_or((1200.0, 800.0));

    let (top, left) = calc_popup_position(&hovered_rect, &hover_position, initial_scroll, window_w, window_h);

    let transform_origin = match hover_position.as_str() {
        "left" => "top left",
        "right" => "top right",
        _ => "top center",
    };

    let title = movie.display_title();
    let backdrop = movie.backdrop_path.clone().or(movie.poster_path.clone());
    let is_tv = movie.is_tv();

    let overview_stored = StoredValue::new(movie.overview.clone());
    let title_stored = StoredValue::new(title);
    let image_url = backdrop.map(|p| {
        if p.starts_with("http") { p } else { format!("{}/w780{}", crate::config::TMDB_IMAGE_URL, p) }
    });
    let image_url_stored = StoredValue::new(image_url);

    let library_store = crate::store::use_library_store();
    let ui_store = crate::store::use_ui_store();
    let movie_stored = StoredValue::new(movie.clone());
    let movie_id = movie.id_u32();
    let is_in_list = Signal::derive(move || {
        library_store.my_list.get().contains_key(&movie_id)
    });

    let (rating, set_rating) = signal(crate::store::ratings::get(movie_id, movie.is_tv()));

    // 1200ms debounce auto-preview trailer
    let (is_playing, set_is_playing) = signal(false);
    let (is_frame_ready, set_is_frame_ready) = signal(false);
    let (card_trailer_time, set_card_trailer_time) = signal(0.0);
    let is_muted = ui_store.preview_muted;
    let (video_key, set_video_key) = signal(None::<String>);
    let (is_teaser, set_is_teaser) = signal(false);

    let movie_id = movie.id_u32();
    let is_tv_for_video = is_tv;

    Effect::new(move |_| {
        let mid = movie_id;
        let is_tv_val = is_tv_for_video;
        if mid > 0 {
            leptos::task::spawn_local(async move {
                if let Ok(videos) = crate::services::tmdb::fetch_videos(mid, is_tv_val).await {
                    if let Some((key, teaser)) = crate::services::tmdb::select_best_tmdb_trailer(&videos) {
                        set_video_key.set(Some(key));
                        set_is_teaser.set(teaser);
                    }
                }
            });
        }

        if let Some(window) = web_sys::window() {
            let cb = Closure::once_into_js(move || {
                set_is_playing.set(true);
                if let Some(w) = web_sys::window() {
                    let ready_cb = Closure::once_into_js(move || {
                        set_is_frame_ready.set(true);
                    });
                    let _ = w.set_timeout_with_callback_and_timeout_and_arguments_0(ready_cb.as_ref().unchecked_ref(), 1200);
                }
            });
            let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(cb.as_ref().unchecked_ref(), 1200);
        }
    });

    Effect::new(move |_| {
        if is_frame_ready.get() && !ui_store.preview_muted.get() {
            if let Some(win) = web_sys::window() {
                if let Some(doc) = win.document() {
                    if let Some(el) = doc.get_element_by_id("card-trailer-iframe") {
                        if let Ok(iframe) = el.dyn_into::<web_sys::HtmlIFrameElement>() {
                            if let Some(cw) = iframe.content_window() {
                                let cmd = r#"{"event":"command","func":"unMute","args":[]}"#;
                                let _ = cw.post_message(&wasm_bindgen::JsValue::from_str(cmd), "*");
                            }
                        }
                    }
                }
            }
        }
    });

    view! {
        <Portal>
            <div
                data-popup="true"
                class="absolute z-(--z-screen) pointer-events-auto rounded-chip overflow-hidden bg-raised text-ink shadow-chip animate-in fade-in zoom-in-95 duration-(--motion-base)"
                style=format!(
                    "width: {}px; top: {}px; left: {}px; transform-origin: {};",
                    popup_w(hovered_rect.width()), top, left, transform_origin
                )
                on:mouseenter=move |_| on_mouse_enter.run(())
                on:mouseleave=move |_| on_mouse_leave.run(())
            >
                // Media / Thumbnail preview
                <div class="relative w-full aspect-video bg-card-alt overflow-hidden">
                    // Base Image layer (always present so no flash of red play button or black box)
                    {
                        let src_opt = image_url_stored.get_value();
                        let t = title_stored.get_value();
                        view! {
                            {if let Some(src) = src_opt {
                                view! {
                                    <img
                                        src=src
                                        alt=t.clone()
                                        class="w-full h-full object-cover"
                                    />
                                }.into_any()
                            } else {
                                view! {
                                    <div class="w-full h-full flex items-center justify-center text-ink/30 text-xs">
                                        {crate::i18n::t("info-modal-episodes-no-image")}
                                    </div>
                                }.into_any()
                            }}
                            <div class="absolute inset-0 bg-gradient-to-t from-raised via-transparent to-transparent opacity-80 pointer-events-none" />
                            <span class="absolute bottom-2 left-3 font-black text-sm tracking-wide drop-shadow-md line-clamp-1 pointer-events-none">
                                {t}
                            </span>
                        }
                    }

                    // Video layer
                    <TrailerPlayer
                        video_key=Signal::derive(move || video_key.get())
                        is_teaser=Signal::derive(move || is_teaser.get())
                        is_playing=Signal::derive(move || is_playing.get())
                        is_muted=ui_store.preview_muted
                        variant="card".to_string()
                        iframe_id="card-trailer-iframe".to_string()
                        on_ready=Callback::new(move |_| {
                            set_is_frame_ready.set(true);
                        })
                        on_ended=Callback::new(move |_| {
                            set_is_playing.set(false);
                        })
                        on_time_update=Callback::new(move |t| {
                            set_card_trailer_time.set(t);
                        })
                    />

                    // Mute toggle button
                    {move || {
                        if video_key.get().is_some() && is_playing.get() {
                            view! {
                                <MuteReplayButton
                                    is_muted=is_muted
                                    is_ended=Signal::derive(|| false)
                                    on_toggle=Callback::new(move |_| {
                                        let new_muted = !ui_store.preview_muted.get_untracked();
                                        ui_store.set_preview_muted(new_muted);
                                    })
                                    size="md".to_string()
                                    class="absolute bottom-3 right-3 z-50 pointer-events-auto".to_string()
                                />
                            }.into_any()
                        } else {
                            view! { <span /> }.into_any()
                        }
                    }}
                </div>

                // Metadata & Action Controls
                <div class="p-4">
                    // Action Buttons Row
                    <div class="flex items-center justify-between min-h-[46px] mb-2">
                        <div class="flex items-center gap-2">
                            // Play button
                            <PlayCircleButton
                                on_click=on_play
                                is_resume=Signal::derive(move || {
                                    let watch_store = crate::store::use_watch_store();
                                    if let Some(rec) = watch_store.get_record(movie_id, is_tv) {
                                        rec.percentage > 0.0
                                    } else {
                                        false
                                    }
                                })
                                size="md".to_string()
                            />

                            // Continue Watching: mark as watched and remove from the row (both take the title out of the row); otherwise Add to My List
                            {match resume.clone() {
                                Some(r) => {
                                    let (done, gone) = (r.on_remove, r.on_remove);
                                    view! {
                                        <crate::components::media::action_buttons::CircleIconButton icon="ph-check" on_click=done title=crate::i18n::t("continue-watching-mark-watched") />
                                        <crate::components::media::action_buttons::CircleIconButton icon="ph-x" on_click=gone title=crate::i18n::t("continue-watching-options-sheet-remove-from-continue-watching") />
                                    }.into_any()
                                }
                                None => view! {
                                    <MyListButton
                                        is_in_list=is_in_list
                                        on_toggle=Callback::new(move |_| {
                                            let m = movie_stored.get_value();
                                            library_store.my_list.update(|map| {
                                                if map.contains_key(&movie_id) {
                                                    map.remove(&movie_id);
                                                } else {
                                                    let entry = crate::store::LibraryEntry {
                                                        media: crate::services::tmdb::MediaItem {
                                                            id: movie_id,
                                                            title: m.title.clone(),
                                                            name: m.name.clone(),
                                                            overview: m.overview.clone(),
                                                            poster_path: m.poster_path.clone(),
                                                            backdrop_path: m.backdrop_path.clone(),
                                                            vote_average: m.vote_average,
                                                            release_date: m.release_date.clone(),
                                                            first_air_date: m.first_air_date.clone(),
                                                            media_type: m.media_type.clone(),
                                                            ..Default::default()
                                                        },
                                                        added_at: js_sys::Date::now() as u64,
                                                    };
                                                    map.insert(movie_id, entry);
                                                }
                                            });
                                        })
                                        size="md".to_string()
                                    />
                                }.into_any(),
                            }}

                            // Rating Pill button
                            <RatingPill
                                rating=rating
                                on_set=Callback::new(move |r: Option<MovieRating>| {
                                    set_rating.set(r);
                                    crate::store::ratings::set(movie_id, movie_stored.get_value().is_tv(), r);
                                })
                                size="md".to_string()
                            />
                        </div>

                        // Expand Info Modal caret
                        <MoreInfoCaretButton
                            on_open=Callback::new(move |_| {
                                let cur_t = card_trailer_time.get_untracked();
                                if cur_t > 0.0 {
                                    ui_store.modal_initial_time.set(cur_t);
                                    ui_store.modal_current_time.set(cur_t);
                                } else {
                                    ui_store.modal_initial_time.set(0.0);
                                    ui_store.modal_current_time.set(0.0);
                                }
                                on_open_modal.run(());
                            })
                            size="md".to_string()
                        />
                    </div>

                    {match resume.clone() {
                        Some(r) => view! {
                            <div class="mt-3 mb-1">
                                <p class="text-ink text-title-sm font-bold leading-snug">{r.label.clone()}</p>
                                <div class="mt-3 flex items-center gap-3">
                                    <div class="h-[3px] flex-1 overflow-hidden rounded-full bg-ink/40">
                                        <div class="h-full bg-brand" style=format!("width: {:.1}%;", r.percent.clamp(2.0, 100.0)) />
                                    </div>
                                    <span class="text-ink-silver text-body-md tabular-nums">{r.time_text.clone()}</span>
                                </div>
                            </div>
                        }.into_any(),
                        None => view! {
                    // Badges & Details Row
                    <div class="flex items-center gap-2 my-[12.8px] min-h-[32px] text-body-md font-normal text-ink-silver flex-wrap">
                        <span class="text-success font-bold">
                            {crate::i18n::t_args("media-percent-match", &[("pct", fluent_bundle::FluentValue::from(movie_stored.get_value().match_score() as i64))])}
                        </span>
                        <RatingBadge id=movie_stored.get_value().id_u32() is_tv=movie_stored.get_value().is_tv() gb=movie_stored.get_value().rating_gb.clone() us=movie_stored.get_value().rating_us.clone() size="xs".to_string() />
                        <span>
                            {if is_tv { crate::i18n::t("media-series") } else { crate::i18n::t("media-film") }}
                        </span>
                        <span class="border border-ink/40 rounded-tick px-[5.6px] text-caption-sm leading-[16px] text-ink/90">
                            {crate::i18n::t("movie-card-popup-hd")}
                        </span>
                    </div>
                        }.into_any(),
                    }}

                    // 3-word vibe descriptors (replacing overview paragraph); not on a Continue Watching card
                    {
                        let is_resume = resume.is_some();
                        let m = movie_stored.get_value();
                        let vibe_tags = derive_vibe_tags(
                            m.vibe_pills.as_deref(),
                            m.genre_ids.as_deref(),
                            &overview_stored.get_value(),
                        );
                        if is_resume { view! { <span /> }.into_any() } else { view! {
                            <div class="flex items-center text-body-md leading-[20.8px] font-normal text-ink flex-wrap mb-2 min-h-[24px]">
                                {vibe_tags.into_iter().enumerate().map(|(idx, tag)| {
                                    view! {
                                        {if idx > 0 {
                                            view! { <span class="text-ink/40 text-body-md px-[3px]">"•"</span> }.into_any()
                                        } else {
                                            view! { <span /> }.into_any()
                                        }}
                                        <span>{crate::i18n::mood_label(&tag)}</span>
                                    }
                                }).collect_view()}
                            </div>
                        }.into_any() }
                    }
                </div>
            </div>
        </Portal>
    }
}
