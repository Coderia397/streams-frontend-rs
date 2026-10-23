use leptos::prelude::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use super::movie_card::MovieCard;

fn set_timeout_ms<F: FnOnce() + 'static>(cb: F, ms: i32) {
    if let Some(window) = web_sys::window() {
        let closure = Closure::once_into_js(cb);
        let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(closure.as_ref().unchecked_ref(), ms);
    }
}

#[component]
pub fn Row(
    #[prop(into)] title: String,
    #[prop(into, optional)] genre_id: Option<String>,
    #[prop(into, optional, default = "movie".to_string())] kind: String,
    #[prop(into, optional)] sort_by: Option<String>,
    #[prop(into, optional)] extra_params: Option<String>,
    #[prop(into, optional)] endpoint: Option<String>,
) -> impl IntoView {
    let (movies, set_movies) = signal::<Vec<crate::services::tmdb::MediaItem>>(Vec::new());
    let (page, set_page) = signal(1);
    let (has_more, set_has_more) = signal(true);
    let (is_fetching, set_is_fetching) = signal(false);

    let genre_id_stored = StoredValue::new(genre_id);
    let kind_stored = StoredValue::new(kind);
    let sort_by_stored = StoredValue::new(sort_by);
    let extra_params_stored = StoredValue::new(extra_params);
    let endpoint_stored = StoredValue::new(endpoint);

    let load_more = move || {
        if is_fetching.get_untracked() || !has_more.get_untracked() {
            return;
        }
        set_is_fetching.set(true);
        let current_page = page.get_untracked();
        let genre_id = genre_id_stored.get_value();
        let kind = kind_stored.get_value();
        let sort_by = sort_by_stored.get_value();
        let extra_params = extra_params_stored.get_value();
        let endpoint = endpoint_stored.get_value();
        
        leptos::task::spawn_local(async move {
            let res = crate::services::tmdb::fetch_row_content(
                &kind,
                genre_id.as_deref(),
                sort_by.as_deref(),
                extra_params.as_deref(),
                endpoint.as_deref(),
                current_page,
            ).await;
            let new_movies = res.unwrap_or_default();
            if new_movies.is_empty() {
                set_has_more.set(false);
            } else {
                // Filter out duplicates (if any) just in case
                set_movies.update(|m| {
                    for movie in new_movies {
                        if !m.iter().any(|existing| existing.id == movie.id) {
                            m.push(movie);
                        }
                    }
                });
                set_page.update(|p| *p += 1);
            }
            set_is_fetching.set(false);
        });
    };

    // Initial load - run only once
    let initial_load_done = StoredValue::new(false);
    Effect::new(move |_| {
        if !initial_load_done.get_value() {
            initial_load_done.set_value(true);
            load_more();
        }
    });

    let scroll_ref = NodeRef::<leptos::html::Div>::new();
    let (can_scroll_left, set_can_scroll_left) = signal(false);
    let (can_scroll_right, set_can_scroll_right) = signal(true);
    // the first page turn starts the loop: from then on the row can also be paged back from its start, to its end
    let (looping, set_looping) = signal(false);
    let (page_index, set_page_index) = signal(0usize);

    let update_scroll_state = Callback::new(move |_: ()| {
        if let Some(el) = scroll_ref.get() {
            let cur = el.scroll_left() as f64;
            let client_w = el.client_width() as f64;
            let scroll_w = el.scroll_width() as f64;

            set_can_scroll_left.set(cur > 10.0);
            set_can_scroll_right.set(scroll_w > client_w + 10.0);

            let margin = crate::utils::layout::app_x();
            let page_w = (client_w - 2.0 * margin + 6.0).max(100.0);
            let idx = ((cur + page_w * 0.3) / page_w).floor() as usize;
            set_page_index.set(idx);
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
            let _ = movies.get();
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
                set_looping.set(true);
                let client_width = el.client_width() as f64;
                let current_scroll = el.scroll_left() as f64;
                let scroll_width = el.scroll_width() as f64;
                
                let margin = crate::utils::layout::app_x();
                let step = (client_width - 2.0 * margin + 6.0).max(100.0);
                
                let target = crate::components::media::row_scroll::page_target(direction != "left", current_scroll, client_width, scroll_width, step);

                el.scroll_to_with_x_and_y(target, 0.0);

                // Proactively load next page if we're getting close to the end
                if direction == "right" && (current_scroll + client_width * 2.0) >= scroll_width {
                    load_more();
                }

                let update_c = update.clone();
                set_timeout_ms(move || {
                    update_c.run(());
                }, 600);
            }
        }
    };

    let card_width_class = "tile-width";

    view! {
        <div class="group relative row-block z-10">
            <div class="flex items-center justify-between px-[var(--app-x,56px)]">
                <h2 class="row-title cursor-pointer flex items-center group/title w-fit">
                    {title}
                    <span class="text-xs text-cyan-500 ml-2 opacity-0 group-hover/title:opacity-100 transition-opacity duration-(--motion-slow) flex items-center font-semibold">
                        {crate::i18n::t("row-explore-all")}
                    </span>
                </h2>

                // Netflix Pagination indicators (dashes) on hover
                {move || {
                    let count = movies.get().len();
                    if count > 4 {
                        let pages = (count / 4).clamp(2, 6);
                        let current = page_index.get();
                        view! {
                            <div class="hidden md:flex items-center gap-[3px] opacity-0 group-hover:opacity-100 transition-opacity duration-(--motion-slow)">
                                {(0..pages).map(|i| {
                                    let is_active = i == current;
                                    view! {
                                        <div
                                            class=if is_active {
                                                "h-[2px] w-3 rounded-full bg-ink transition-colors duration-(--motion-base)"
                                            } else {
                                                "h-[2px] w-3 rounded-full bg-ink/20 transition-colors duration-(--motion-base)"
                                            }
                                        />
                                    }
                                }).collect::<Vec<_>>()}
                            </div>
                        }.into_any()
                    } else {
                        view! { <span /> }.into_any()
                    }
                }}
            </div>

            <div class="relative group/row row-scroll-outer" style="container-type: inline-size;">
                {move || {
                    let m = movies.get();
                    if m.is_empty() && is_fetching.get() {
                        view! {
                            <div class="flex overflow-x-scroll scrollbar-hide w-full pointer-events-auto relative z-10 py-[2px] gap-[var(--tile-gap)]">
                                <div class="flex-none h-full pointer-events-none" style="width: var(--app-x, 56px);" />
                                {(0..6).map(|_| view! {
                                    <div
                                        class=format!("movie-card-container relative flex-none {} aspect-video bg-card rounded-tile overflow-hidden border border-ink/[0.04] pointer-events-auto", card_width_class)
                                    >
                                        <div class="absolute inset-0 -translate-x-full animate-[shimmer_1.8s_ease-in-out_infinite] bg-gradient-to-r from-transparent via-ink/[0.05] to-transparent" />
                                        <div class="absolute inset-0 bg-gradient-to-b from-field via-card to-raised" />
                                        <div class="absolute bottom-4 left-3 space-y-2">
                                            <div class="h-2.5 bg-ink/[0.08] rounded-full w-16" />
                                            <div class="h-1.5 bg-ink/[0.05] rounded-full w-10" />
                                        </div>
                                    </div>
                                }).collect::<Vec<_>>()}
                                <div class="flex-none h-full pointer-events-none" style="width: var(--app-x, 56px);" />
                            </div>
                        }.into_any()
                    } else if m.is_empty() {
                        view! { <div /> }.into_any()
                    } else {
                        let scroll_left = scroll.clone();
                        let scroll_right = scroll.clone();
                        
                        view! {
                            <>
                                <div
                                    node_ref=scroll_ref
                                    on:scroll=on_scroll
                                    class="row-scroll-strip flex overflow-x-scroll scrollbar-hide w-full pointer-events-auto relative z-10 py-[2px] gap-[var(--tile-gap)]"
                                    style="scroll-behavior: smooth;"
                                >
                                    <div class="flex-none h-full pointer-events-none" style="width: var(--app-x, 56px);" />

                                    {m.into_iter().map(|item| {
                                        let movie_id = item.id;
                                        let is_tv = item.is_tv();
                                        let title = item.display_title().to_string();
                                        let backdrop = item.backdrop_url("w780").unwrap_or_default();
                                        let poster = item.poster_url("w342").unwrap_or_default();
                                        
                                        view! {
                                            <div 
                                                class=format!("movie-card-container relative flex-none pointer-events-auto overflow-visible rounded-tile {} aspect-video", card_width_class)
                                                style="z-index: auto;"
                                            >
                                                <MovieCard
                                                    movie_id=movie_id
                                                    is_tv=is_tv
                                                    title=title
                                                    backdrop_path=backdrop
                                                    poster_path=poster
                                                    vote_average=item.vote_average
                                                    overview=item.overview.clone()
                                                    genre_ids=item.genre_ids.clone()
                                                    vibe_pills=item.vibe_pills.clone()
                                                />
                                            </div>
                                        }
                                    }).collect::<Vec<_>>()}
                                    
                                    {move || if is_fetching.get() {
                                        view! {
                                            <div class=format!("movie-card-container relative flex-none {} aspect-video flex items-center justify-center", card_width_class)>
                                                <div class="w-8 h-8 rounded-full border-2 border-transparent border-t-ink/60 animate-spin" />
                                            </div>
                                        }.into_any()
                                    } else {
                                        view! { <div /> }.into_any()
                                    }}

                                    <div class="flex-none h-full pointer-events-none" style="width: var(--app-x, 56px);" />
                                </div>

                                <crate::components::media::row_scroll::EdgeButton edge=crate::components::media::row_scroll::Edge::Left visible=Signal::derive(move || can_scroll_left.get() || looping.get()) on_click=Callback::new(move |_| scroll_left("left")) label=crate::i18n::t("continue-watching-row-scroll-left") />
                                <crate::components::media::row_scroll::EdgeButton edge=crate::components::media::row_scroll::Edge::Right visible=can_scroll_right on_click=Callback::new(move |_| scroll_right("right")) label=crate::i18n::t("continue-watching-row-scroll-right") />
                            </>
                        }.into_any()
                    }
                }}
            </div>
        </div>
    }
}
