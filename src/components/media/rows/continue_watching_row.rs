use leptos::prelude::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use leptos_router::hooks::use_navigate;
use crate::store::use_watch_store;
use crate::store::use_ui_store;

fn set_timeout_ms<F: FnOnce() + 'static>(cb: F, ms: i32) {
    if let Some(window) = web_sys::window() {
        let closure = Closure::once_into_js(cb);
        let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(closure.as_ref().unchecked_ref(), ms);
    }
}

#[component]
pub fn ContinueWatchingRow(
    /// A heading of its own; the home page shows "Continue watching for <name>".
    #[prop(optional, into)] title: Option<String>,
) -> impl IntoView {
    let title = StoredValue::new(title);
    let watch_store = use_watch_store();
    let ui_store = use_ui_store();
    let navigate = use_navigate();
    let scroll_ref = NodeRef::<leptos::html::Div>::new();
    let (can_scroll_left, set_can_scroll_left) = signal(false);
    let (can_scroll_right, set_can_scroll_right) = signal(true);

    let update_scroll_state = Callback::new(move |_: ()| {
        if let Some(el) = scroll_ref.get_untracked() {
            let cur = el.scroll_left() as f64;
            let client_w = el.client_width() as f64;
            let scroll_w = el.scroll_width() as f64;

            set_can_scroll_left.set(cur > 10.0);
            set_can_scroll_right.set(scroll_w > client_w + 10.0 && scroll_w - (cur + client_w) > 10.0);
        }
    });

    let on_scroll = {
        let update = update_scroll_state.clone();
        move |_| update.run(())
    };

    // Window resize listener
    {
        let update = update_scroll_state.clone();
        crate::utils::listen::listen_window(leptos::ev::resize, move |_| update.run(()));
    }

    // Content change / initial render listener
    Effect::new({
        let update = update_scroll_state.clone();
        move |_| {
            let _ = watch_store.get_continue_watching_list();
            let update_c = update.clone();
            set_timeout_ms(move || {
                update_c.run(());
            }, 100);
        }
    });

    let scroll = {
        let update = update_scroll_state.clone();
        move |direction: &str| {
            if let Some(el) = scroll_ref.get() {
                let client_width = el.client_width() as f64;
                let current_scroll = el.scroll_left() as f64;
                let scroll_width = el.scroll_width() as f64;
                
                let margin = if client_width >= 1750.0 { 60.0 } else if client_width >= 1350.0 { 56.0 } else if client_width >= 800.0 { 48.0 } else { 16.0 };
                let step = (client_width - 2.0 * margin + 6.0).max(100.0);
                
                let target = if direction == "left" {
                    (current_scroll - step).max(0.0)
                } else {
                    (current_scroll + step).min(scroll_width - client_width)
                };
                el.scroll_to_with_x_and_y(target, 0.0);

                let update_c = update.clone();
                set_timeout_ms(move || {
                    update_c.run(());
                }, 350);
            }
        }
    };

    let card_width_class = "tile-width tile-landscape";

    view! {
        {move || {
            let items = watch_store.get_continue_watching_list();
            if items.is_empty() {
                view! { <span /> }.into_any()
            } else {
                let scroll_left = scroll.clone();
                let scroll_right = scroll.clone();

                view! {
                    <div class="group relative row-block z-10">
                        <div class="flex items-center justify-between px-[var(--app-x,56px)]">
                            <h2 class="row-title cursor-pointer flex items-center group/title w-fit">
                                {move || {
                                    if let Some(t) = title.get_value() { return t; }
                                    let pstore = crate::store::use_profile_store();
                                    let name = pstore.active_profile_id.get().and_then(|id| pstore.profiles.get().into_iter().find(|p| p.id == id)).map(|p| p.name).filter(|n| !n.trim().is_empty());
                                    match name {
                                        Some(n) => crate::i18n::t_args("continue-watching-row-for", &[("name", fluent_bundle::FluentValue::from(n))]),
                                        None => crate::i18n::t("continue-watching-row-continue-watching"),
                                    }
                                }}
                            </h2>
                        </div>

                        <div class="relative group/row row-scroll-outer" style="container-type: inline-size;">
                            <div
                                node_ref=scroll_ref
                                on:scroll=on_scroll
                                class="row-scroll-strip flex overflow-x-scroll scrollbar-hide w-full pointer-events-auto relative z-10 py-[2px] gap-[var(--tile-gap)]"
                                style="scroll-behavior: smooth;"
                            >
                                <div class="row-edge" />

                                {items.into_iter().map(|rec| {
                                    let nav_play = navigate.clone();
                                    let id = rec.media_id;
                                    let is_tv = rec.is_tv;
                                    let season = rec.season;
                                    let episode = rec.episode;
                                    let title = rec.title.clone();
                                    let episode_title = rec.episode_title.clone();
                                    let percentage = rec.percentage.clamp(0.0, 100.0);
                                    let backdrop = rec.backdrop_url("w780").or_else(|| rec.poster_url("w500")).unwrap_or_default();

                                    let target_url = if is_tv {
                                        if let (Some(s), Some(e)) = (season, episode) {
                                            format!("/watch/tv/{}?season={}&episode={}", id, s, e)
                                        } else {
                                            format!("/watch/tv/{}", id)
                                        }
                                    } else {
                                        format!("/watch/movie/{}", id)
                                    };

                                    let target_url_for_play = target_url.clone();
                                    let handle_play = move |e: leptos::ev::MouseEvent| {
                                        e.stop_propagation();
                                        nav_play(&target_url_for_play, Default::default());
                                    };

                                    let handle_open_info = move |e: leptos::ev::MouseEvent| {
                                        e.stop_propagation();
                                        ui_store.hero_paused_by_modal.set(false);
                                        ui_store.modal_initial_time.set(0.0);
                                        ui_store.modal_current_time.set(0.0);
                                        ui_store.info_modal_movie_id.set(Some(id));
                                        ui_store.info_modal_is_tv.set(is_tv);
                                        ui_store.info_modal_open.set(true);
                                    };

                                    let handle_remove = move |e: leptos::ev::MouseEvent| {
                                        e.stop_propagation();
                                        watch_store.remove_record(id, is_tv);
                                    };

                                    let ep_badge = if is_tv {
                                        match (season, episode) {
                                            (Some(s), Some(e)) => Some(format!("S{}:E{}", s, e)),
                                            _ => None,
                                        }
                                    } else {
                                        None
                                    };

                                    let pct_label = crate::i18n::t_args("media-percent-watched", &[("pct", fluent_bundle::FluentValue::from(format!("{:.0}", percentage)))]);
                                    // the label of the hover card: the episode of a series (`S1:E22 "Blood Brothers"`), the title of a film; and how far: `16 of 43m`
                                    let label = match (is_tv, season, episode, episode_title.clone()) {
                                        (true, Some(s), Some(e), Some(t)) if !t.is_empty() => format!("S{}:E{} \"{}\"", s, e, t),
                                        (true, Some(s), Some(e), _) => format!("S{}:E{}", s, e),
                                        _ => title.clone(),
                                    };
                                    let time_text = crate::i18n::t_args("continue-watching-time-of", &[
                                        ("done", fluent_bundle::FluentValue::from((rec.current_time / 60.0).round() as i64)),
                                        ("total", fluent_bundle::FluentValue::from((rec.duration / 60.0).round() as i64)),
                                    ]);
                                    let resume = crate::components::media::movie_card_popup::ResumeInfo {
                                        label,
                                        time_text,
                                        percent: percentage,
                                        on_remove: Callback::new(move |_| watch_store.remove_record(id, is_tv)),
                                    };
                                    let poster = rec.poster_url("w342").unwrap_or_default();
                                    let _ = (&handle_open_info, &handle_remove, &handle_play, &ep_badge);
                                    view! {
                                        <div class=format!("relative flex-none {}", card_width_class)>
                                            // The picture and its hover card (the same tile as everywhere else)
                                            <div class="movie-card-container relative pointer-events-auto overflow-visible rounded-tile w-full aspect-video" style="z-index: auto;">
                                                <crate::components::media::movie_card::MovieCard
                                                    movie_id=id
                                                    is_tv=is_tv
                                                    title=title.clone()
                                                    backdrop_path=backdrop
                                                    poster_path=poster
                                                    vote_average=0.0
                                                    resume=Some(resume)
                                                />
                                            </div>

                                            // How far you are: a short bar under the picture (Netflix's: 60 % of the card, 3 px, 5 px under it), the brand colour for what is watched and white for what is left
                                            <div class="mx-auto mt-[5px] h-[3px] w-[60%] flex overflow-hidden rounded-full bg-ink" aria-label=pct_label.clone()>
                                                <div class="h-full bg-brand" style=format!("width: {:.1}%;", percentage.clamp(2.0, 100.0)) />
                                            </div>
                                        </div>
                                    }
                                }).collect::<Vec<_>>()}

                                <div class="row-edge" />
                            </div>

                            // Left Chevron
                            <button
                                type="button"
                                class=move || if can_scroll_left.get() {
                                    "hidden md:flex absolute top-0 bottom-4 left-0 z-30 items-center justify-center cursor-pointer bg-transparent hover:bg-void/70 transition-all duration-(--motion-base) opacity-0 group-hover/row:opacity-100 group-hover/row:pointer-events-auto select-none border-none outline-none"
                                } else {
                                    "hidden !pointer-events-none !opacity-0"
                                }
                                style="width: var(--app-x, 56px);"
                                on:click=move |_| scroll_left("left")
                                aria-label=crate::i18n::t("continue-watching-row-scroll-left")
                            >
                                <i class="ph-bold ph-caret-left text-ink text-3xl sm:text-4xl drop-shadow-lg transition-transform hover:scale-125"></i>
                            </button>

                            // Right Chevron
                            <button
                                type="button"
                                class=move || if can_scroll_right.get() {
                                    "hidden md:flex absolute top-0 bottom-4 right-0 z-30 items-center justify-center cursor-pointer bg-transparent hover:bg-void/70 transition-all duration-(--motion-base) opacity-0 group-hover/row:opacity-100 group-hover/row:pointer-events-auto select-none border-none outline-none"
                                } else {
                                    "hidden !pointer-events-none !opacity-0"
                                }
                                style="width: var(--app-x, 56px);"
                                on:click=move |_| scroll_right("right")
                                aria-label=crate::i18n::t("continue-watching-row-scroll-right")
                            >
                                <i class="ph-bold ph-caret-right text-ink text-3xl sm:text-4xl drop-shadow-lg transition-transform hover:scale-125"></i>
                            </button>
                        </div>
                    </div>
                }.into_any()
            }
        }}
    }
}
