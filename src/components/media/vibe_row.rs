use leptos::prelude::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use crate::services::tmdb::MediaItem;
use crate::services::ai_engine::RowMore;
use super::movie_card::MovieCard;

/// Titles asked for at a time once a row is paged near the end of what it shows.
const MORE_ITEMS: usize = 48;

fn set_timeout_ms<F: FnOnce() + 'static>(cb: F, ms: i32) {
    if let Some(window) = web_sys::window() {
        let closure = Closure::once_into_js(cb);
        let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(closure.as_ref().unchecked_ref(), ms);
    }
}

#[component]
pub fn VibeRow(
    #[prop(into)] title: String,
    #[prop(default = None)] tagline: Option<String>,
    #[prop(optional, default = Vec::new())] mood_pills: Vec<String>,
    #[prop(into)] items: Vec<MediaItem>,
    /// Where the rest of the row's titles come from, when the page sent only its first ones (the engine keeps rows of up to 150 titles, see `/api/feed/row`)
    #[prop(optional_no_strip)] more: Option<RowMore>,
) -> impl IntoView {
    let _ = (&tagline, &mood_pills);
    let scroll_ref = NodeRef::<leptos::html::Div>::new();
    let (can_scroll_left, set_can_scroll_left) = signal(false);
    let (can_scroll_right, set_can_scroll_right) = signal(true);
    // the first page turn starts the loop: from then on the row can also be paged back from its start, to its end
    let (looping, set_looping) = signal(false);
    let (page_index, set_page_index) = signal(0);
    // how many pages the row has once it is on the screen (0 until measured): the dashes follow it
    let (page_count, set_page_count) = signal(0usize);

    // the titles on the strip: the page's first ones, then what the row fetches as it is paged towards its end
    let items_sig = RwSignal::new(items);
    let more_src = StoredValue::new(more);
    let (has_more, set_has_more) = signal(more_src.with_value(|m| m.is_some()));
    let (loading, set_loading) = signal(false);
    let load_more = Callback::new(move |_: ()| {
        if loading.get_untracked() || !has_more.get_untracked() {
            return;
        }
        let Some(src) = more_src.get_value() else { return };
        set_loading.set(true);
        let offset = items_sig.with_untracked(|v| v.len());
        wasm_bindgen_futures::spawn_local(async move {
            match crate::services::ai_engine::fetch_row_items(&src.query, &src.row_id, offset, MORE_ITEMS).await {
                Some(page) => {
                    let fresh: Vec<MediaItem> = page.items.iter().map(|i| i.to_media_item()).collect();
                    let got = !fresh.is_empty();
                    items_sig.update(|v| v.extend(fresh));
                    set_has_more.set(page.has_more_items && got);
                }
                None => set_has_more.set(false),
            }
            set_loading.set(false);
        });
    });

    let update_scroll_state = Callback::new(move |_: ()| {
        if let Some(el) = scroll_ref.get() {
            let cur = el.scroll_left() as f64;
            let client_w = el.client_width() as f64;
            let scroll_w = el.scroll_width() as f64;
            set_can_scroll_left.set(cur > 10.0);
            set_can_scroll_right.set(scroll_w > client_w + 10.0);

            if let Some((i, n)) = crate::components::media::row_scroll::page_position(&el, crate::utils::layout::app_x()) {
                set_page_index.set(i);
                set_page_count.set(n);
            } else if client_w > 0.0 {
                let p = (cur / (client_w * 0.85)).round() as usize;
                set_page_index.set(p);
            }
            // within a screen and a half of the end of what the strip shows: ask for the next titles before the viewer gets there
            if has_more.get_untracked() && scroll_w - (cur + client_w) < client_w * 1.5 {
                load_more.run(());
            }
        }
    });

    {
        let update = update_scroll_state.clone();
        crate::utils::listen::listen_window(leptos::ev::resize, move |_| update.run(()));
    }

    let on_scroll = {
        let update = update_scroll_state.clone();
        move |_| update.run(())
    };

    let scroll = {
        let update = update_scroll_state.clone();
        move |direction: &'static str| {
            if let Some(el) = scroll_ref.get() {
                set_looping.set(true);
                let client_width = el.client_width() as f64;
                let current_scroll = el.scroll_left() as f64;
                let scroll_width = el.scroll_width() as f64;
                
                let margin = if client_width >= 1750.0 { 60.0 } else if client_width >= 1350.0 { 56.0 } else if client_width >= 800.0 { 48.0 } else { 16.0 };
                let step = (client_width - 2.0 * margin + 6.0).max(100.0);
                
                if crate::components::media::row_scroll::turn_page(&el, direction != "left", crate::utils::layout::app_x()) {
                    let update_c = update.clone();
                    set_timeout_ms(move || { update_c.run(()); }, 600);
                    return;
                }
                let target = crate::components::media::row_scroll::page_target(direction != "left", current_scroll, client_width, scroll_width, step);

                el.scroll_to_with_x_and_y(target, 0.0);

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
            // Row Header: Title + Pagination Indicators
            <div class="flex items-center justify-between px-[var(--app-x,56px)]">
                <h2 class="row-title flex items-center w-fit">
                    {title.clone()}
                </h2>

                // Netflix Pagination indicators (dashes) on hover
                {move || {
                    let count = items_sig.with(|it| it.len());
                    if count > 4 {
                        let pages = if page_count.get() > 0 { page_count.get() } else { (count / 4).clamp(2, 6) };
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

            // Horizontal Carousel Strip
            <div class="relative group/row row-scroll-outer" style="container-type: inline-size;">
                {
                    if items_sig.with_untracked(|m| m.is_empty()) {
                        view! { <div /> }.into_any()
                    } else {
                        let scroll_left = scroll.clone();
                        let scroll_right = scroll.clone();
                        let card = move |item: crate::services::tmdb::MediaItem, role: u8| {
                            let movie_id = item.id;
                            let is_tv = item.is_tv();
                            let title_item = item.display_title().to_string();
                            let backdrop = item.backdrop_url("w780").unwrap_or_default();
                            let poster = item.poster_url("w342").unwrap_or_default();
                            let vote_avg = item.vote_average;
                            let match_pct = item.match_percentage;
                            let vibe_pills = item.vibe_pills.clone();
                            let overview = item.overview.clone();
                            let genre_ids = item.genre_ids.clone();
                            let is_new = item.badges.iter().any(|b| b == "new");
                            view! {
                                <div
                                    class=format!("movie-card-container relative flex-none pointer-events-auto overflow-visible rounded-tile {} aspect-video{}", card_width_class, if role >= 2 { " max-md:hidden" } else { "" })
                                    style="z-index: auto;"
                                    data-loop-first=(role == 1).then_some("1")
                                    data-loop-clone=(role == 2).then_some("1")
                                >
                                    <MovieCard
                                        movie_id=movie_id
                                        is_tv=is_tv
                                        title=title_item
                                        backdrop_path=backdrop
                                        poster_path=poster
                                        vote_average=vote_avg
                                        overview=overview
                                        genre_ids=genre_ids
                                        match_percentage=match_pct
                                        vibe_pills=vibe_pills
                                        is_new=is_new
                                    />
                                </div>
                            }
                        };
                        // how many cards are copied at each end so the row can be paged round (see row_scroll.rs): none while more titles are still to come
                        let loop_cards = move |n: usize| if n > 7 && !has_more.get() { n.min(12) } else { 0 };
                        // the copy of the last cards in front of the row, and the cards with a copy of the first ones behind; both rebuilt when more titles arrive
                        let lead_views = move || {
                            let m = items_sig.get();
                            let k = loop_cards(m.len());
                            m[m.len() - k..].iter().cloned().map(|item| card(item, 3)).collect::<Vec<_>>()
                        };
                        let card_views = move || {
                            let m = items_sig.get();
                            let k = loop_cards(m.len());
                            let copies: Vec<_> = m[..k].to_vec();
                            let mut views: Vec<_> = m.into_iter().enumerate().map(|(i, item)| card(item, if i == 0 { 1 } else { 0 })).collect();
                            views.extend(copies.into_iter().enumerate().map(|(i, item)| card(item, if i == 0 { 2 } else { 3 })));
                            views
                        };
                        view! {
                            <>
                                <div
                                    node_ref=scroll_ref
                                    on:scroll=on_scroll
                                    class="row-scroll-strip flex overflow-x-scroll scrollbar-hide w-full pointer-events-auto relative z-10 py-[2px] gap-[var(--tile-gap)]"
                                    style="scroll-behavior: smooth;"
                                >
                                    // the copy of the last cards in front of the row: not shown until the row is paged for the first time (see row_scroll.rs)
                                    <div data-loop-lead="1" class="flex-none gap-[var(--tile-gap)]" style="display: none;">{lead_views}</div>
                                    <div data-loop-spacer="1" class="flex-none h-full pointer-events-none" style="width: var(--app-x, 56px);" />

                                    {card_views}

                                    <div class="flex-none h-full pointer-events-none" style="width: var(--app-x, 56px);" />
                                </div>

                                <crate::components::media::row_scroll::EdgeButton edge=crate::components::media::row_scroll::Edge::Left visible=Signal::derive(move || can_scroll_left.get() || looping.get()) on_click=Callback::new(move |_| scroll_left("left")) label=crate::i18n::t("vibe-row-scroll-left") />
                                <crate::components::media::row_scroll::EdgeButton edge=crate::components::media::row_scroll::Edge::Right visible=can_scroll_right on_click=Callback::new(move |_| scroll_right("right")) label=crate::i18n::t("vibe-row-scroll-right") />
                            </>
                        }.into_any()
                    }
                }
            </div>
        </div>
    }
}
