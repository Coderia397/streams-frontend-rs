use leptos::prelude::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use crate::services::tmdb::{fetch_trending, MediaItem};
use crate::models::movie::Movie;
use crate::store::use_ui_store;
use crate::components::media::movie_card_popup::MovieCardPopup;

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

/// The rank beside a Top 10 poster: the numeral as a shape cut from the site's font (`top_ten_numerals`), so it is the same in every browser
/// and never reaches past its box. From a tablet up it fills the left half of the slot, its outline fading from light on the left to dark on
/// the right, tucked under the poster; on phones it is a little under half the poster's height at the bottom left, a thin light outline
/// (style/input.css).
#[component]
fn RankNumber(index: usize) -> impl IntoView {
    let (width, path) = crate::components::media::top_ten_numerals::NUMERALS[index.min(9)];
    let gradient_id = {
        use std::sync::atomic::{AtomicUsize, Ordering};
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        format!("tt-grad-{}-{}", index, NEXT.fetch_add(1, Ordering::Relaxed))
    };
    // built as text so that the gradient keeps its <stop> children (the view macro wraps them in <g>, which empties the gradient); the line
    // is drawn under the fill (`paint-order`), so only its outer half shows and the inner contours of the digits stay hidden
    let svg_markup = format!(
        "<svg class=\"topten-svg\" viewBox=\"0 0 {width} {height}\" preserveAspectRatio=\"xMaxYMax meet\" aria-hidden=\"true\" style=\"--tt-stroke:url(#{gradient_id})\">\
         <defs><linearGradient id=\"{gradient_id}\" x1=\"0\" y1=\"0\" x2=\"1\" y2=\"0\">\
         <stop offset=\"0\" stop-color=\"#aaaaaa\"/><stop offset=\"0.5\" stop-color=\"#666666\"/><stop offset=\"1\" stop-color=\"#262626\"/></linearGradient></defs>\
         <path class=\"topten-num\" d=\"{path}\" stroke-width=\"{stroke}\" stroke-linejoin=\"round\" paint-order=\"stroke\"/></svg>",
        height = crate::components::media::top_ten_numerals::BOX_HEIGHT,
        stroke = crate::components::media::top_ten_numerals::STROKE,
    );

    view! {
        <div class="topten-rank pointer-events-none absolute bottom-0 left-0 top-0 z-0 flex w-1/2 items-center justify-end" aria-hidden="true">
            <div class="topten-svgwrap" inner_html=svg_markup></div>
        </div>
    }
}

#[component]
fn TopTenCard(
    item: MediaItem,
    index: usize,
    is_tv: bool,
    /// 1 for the first card of the row, 2 for the first card of its copy at the end (see row_scroll.rs), 3 for the other cards of the copy (the copy is for the website form only), 0 otherwise
    #[prop(default = 0)] loop_role: u8,
) -> impl IntoView {
    let ui_store = use_ui_store();
    let movie_id = item.id;
    let my_card_id = format!("topten-{}-{}", if is_tv { "tv" } else { "movie" }, movie_id);

    let (is_hovered, set_is_hovered) = signal(false);
    let (rect, set_rect) = signal::<Option<web_sys::DomRect>>(None);
    let (scroll_pos, set_scroll_pos) = signal((0.0, 0.0));
    let (hover_pos, set_hover_pos) = signal("center".to_string());

    let show_timer = StoredValue::new(None::<i32>);
    let close_timer = StoredValue::new(None::<i32>);

    let card_ref = NodeRef::<leptos::html::Div>::new();

    let title_stored = StoredValue::new(item.display_title().to_string());
    let poster_url_stored = StoredValue::new(item.poster_url("w342").unwrap_or_default());
    let backdrop_url_stored = StoredValue::new(item.backdrop_url("w780").unwrap_or_default());
    let overview_stored = StoredValue::new(item.overview.clone());
    let is_new = item.badges.iter().any(|b| b == "new");
    let vote_avg = item.vote_average;
    let match_percentage = item.match_percentage;
    let vibe_pills = item.vibe_pills.clone();
    let genre_ids = item.genre_ids.clone();

    let cancel_close_cb = Callback::new(move |_| {
        clear_timeout_id(close_timer.get_value());
        close_timer.set_value(None);
    });

    let my_card_id_for_leave = my_card_id.clone();
    let mouse_leave_cb = Callback::new(move |_| {
        clear_timeout_id(show_timer.get_value());
        show_timer.set_value(None);

        clear_timeout_id(close_timer.get_value());
        let card_id = my_card_id_for_leave.clone();
        let timer_id = set_timeout_ms(move || {
            close_timer.set_value(None);
            set_is_hovered.set(false);
            set_rect.set(None);
            if ui_store.active_popup_id.get_untracked().as_deref() == Some(&card_id) {
                ui_store.active_popup_id.set(None);
            }
        }, 150);
        close_timer.set_value(timer_id);
    });

    let my_card_id_for_enter = my_card_id.clone();
    let mouse_enter_cb = Callback::new(move |_| {
        cancel_close_cb.run(());

        if is_hovered.get_untracked() {
            return;
        }
        if show_timer.get_value().is_some() {
            return;
        }

        let card_id = my_card_id_for_enter.clone();
        // the time the pointer must rest on the card is used to get what the hover card shows (the age rating), so that it opens complete
        crate::services::rating::prefetch(movie_id, is_tv);
        let timer_id = set_timeout_ms(move || {
            show_timer.set_value(None);
            if let Some(el) = card_ref.get() {
                let r = el.get_bounding_client_rect();
                let win = web_sys::window().unwrap();
                let win_w = win.inner_width().unwrap().as_f64().unwrap_or(1200.0);
                let sx = win.scroll_x().unwrap_or(0.0);
                let sy = win.scroll_y().unwrap_or(0.0);

                let app_x = crate::utils::layout::app_x();
                let half_popup = crate::components::media::movie_card_popup::popup_w(r.width()) / 2.0;
                let card_center = r.left() + r.width() / 2.0;

                let number_pl = r.width();
                let item_left = r.left() - number_pl;

                let pos = if item_left <= app_x + 30.0 || card_center - half_popup < app_x {
                    "left"
                } else if card_center + half_popup > win_w - app_x - 15.0 {
                    "right"
                } else {
                    "center"
                };

                set_hover_pos.set(pos.to_string());
                set_rect.set(Some(r));
                set_scroll_pos.set((sx, sy));
                set_is_hovered.set(true);
                ui_store.active_popup_id.set(Some(card_id));
            }
        }, crate::utils::layout::CARD_HOVER_DELAY_MS);
        show_timer.set_value(timer_id);
    });

    let my_card_id_for_effect = my_card_id.clone();
    Effect::new(move |_| {
        let active = ui_store.active_popup_id.get();
        if is_hovered.get() {
            if let Some(ref id) = active {
                if id != &my_card_id_for_effect {
                    clear_timeout_id(show_timer.get_value());
                    clear_timeout_id(close_timer.get_value());
                    show_timer.set_value(None);
                    close_timer.set_value(None);
                    set_is_hovered.set(false);
                    set_rect.set(None);
                }
            }
        }
    });

    on_cleanup(move || {
        clear_timeout_id(show_timer.get_value());
        clear_timeout_id(close_timer.get_value());
    });

    let open_modal = move |_| {
        set_is_hovered.set(false);
        set_rect.set(None);
        ui_store.active_popup_id.set(None);
        ui_store.hero_paused_by_modal.set(false);
        ui_store.modal_initial_time.set(0.0);
        ui_store.modal_current_time.set(0.0);
        ui_store.info_modal_movie_id.set(Some(movie_id));
        ui_store.info_modal_is_tv.set(is_tv);
        ui_store.info_modal_open.set(true);
    };

    let title_for_img = title_stored.with_value(|t| t.clone());
    let poster_src = poster_url_stored.with_value(|p| p.clone());

    view! {
        <div
            data-card="true"
            data-card-id=my_card_id
            data-loop-first=(loop_role == 1).then_some("1")
            data-loop-clone=(loop_role == 2).then_some("1")
            class=format!("relative flex-none flex items-end tile-width topten-card cursor-pointer{}", if loop_role >= 2 { " max-sm:hidden" } else { "" })
            // only a mouse or a pen opens the hover card, and never on the phone form (form::opens_hover_card)
            on:pointerenter=move |e: leptos::ev::PointerEvent| if crate::form::opens_hover_card(&e) { mouse_enter_cb.run(()) }
            on:pointerleave=move |e: leptos::ev::PointerEvent| if crate::form::opens_hover_card(&e) { mouse_leave_cb.run(()) }
            on:click=open_modal
        >
            <RankNumber index=index />

            // Poster card — card_ref is HERE so popup positions relative to the poster,
            // not the outer div (which includes the wide number padding-left).
            <div
                node_ref=card_ref
                class="topten-poster relative flex-none ml-auto w-1/2 aspect-[7/10] z-10 rounded-l-none rounded-r-[3.6cqw] overflow-hidden">
                <img
                    src=poster_src
                    alt=title_for_img.clone()
                    class="w-full h-full object-cover object-top rounded-l-none rounded-r-[3.6cqw]"
                    loading="lazy"
                    draggable="false"
                />

                // "Recently Added" (the engine's `new` badge): a red tab at the bottom of the poster, centred, as in the app
                {is_new.then(|| view! {
                    <span class="absolute bottom-0 left-1/2 -translate-x-1/2 z-10 whitespace-nowrap rounded-[1.2cqw] bg-brand px-[4.2cqw] py-[1.8cqw] text-[5.4cqw] font-bold leading-none text-chrome-ink pointer-events-none">
                        {crate::i18n::t("badge-recently-added")}
                    </span>
                })}

                // Watch Progress Bar
                {move || {
                    let watch_store = crate::store::use_watch_store();
                    if let Some(rec) = watch_store.get_record(movie_id, is_tv) {
                        if rec.percentage > 0.0 {
                            let pct = rec.percentage.clamp(0.0, 100.0);
                            view! {
                                <div class="absolute bottom-0 inset-x-0 h-1 bg-ink/20 z-10 pointer-events-none">
                                    <div class="h-full bg-brand" style=format!("width: {:.1}%;", pct) />
                                </div>
                            }.into_any()
                        } else {
                            view! { <span /> }.into_any()
                        }
                    } else {
                        view! { <span /> }.into_any()
                    }
                }}
            </div>

            {move || {
                if is_hovered.get() {
                    if let Some(r) = rect.get() {
                        let t = title_stored.with_value(|t| t.clone());
                        let b = backdrop_url_stored.with_value(|b| b.clone());
                        let p = poster_url_stored.with_value(|p| p.clone());
                        let ov = overview_stored.with_value(|ov| ov.clone());
                        let m = Movie {
                            id: serde_json::json!(movie_id),
                            title: if !is_tv { Some(t.clone()) } else { None },
                            name: if is_tv { Some(t) } else { None },
                            backdrop_path: if !b.is_empty() { Some(b) } else { None },
                            poster_path: if !p.is_empty() { Some(p) } else { None },
                            overview: ov,
                            vote_average: vote_avg,
                            media_type: Some(if is_tv { "tv".to_string() } else { "movie".to_string() }),
                            match_percentage,
                            genre_ids: if genre_ids.is_empty() { None } else { Some(genre_ids.iter().map(|&g| g as i64).collect()) },
                            vibe_pills: if vibe_pills.is_empty() { None } else { Some(vibe_pills.clone()) },
                            ..Default::default()
                        };
                        let on_play_cb = Callback::new(move |_| {
                            crate::components::player::open::open_player(&crate::components::player::open::watch_url(movie_id, is_tv));
                        });
                        let on_modal_cb = Callback::new(move |_| {
                            set_is_hovered.set(false);
                            set_rect.set(None);
                            ui_store.active_popup_id.set(None);
                            ui_store.info_modal_movie_id.set(Some(movie_id));
                            ui_store.info_modal_is_tv.set(is_tv);
                            ui_store.info_modal_open.set(true);
                        });
                        view! {
                            <MovieCardPopup
                                movie=m
                                hovered_rect=r
                                initial_scroll=scroll_pos.get()
                                hover_position=hover_pos.get()
                                on_mouse_enter=cancel_close_cb
                                on_mouse_leave=mouse_leave_cb
                                on_play=on_play_cb
                                on_open_modal=on_modal_cb
                            />
                        }.into_any()
                    } else {
                        view! { <span /> }.into_any()
                    }
                } else {
                    view! { <span /> }.into_any()
                }
            }}
        </div>
    }
}

#[component]
pub fn TopTenRow(
    #[prop(into, optional, default = "Top 10 in the UK Today".to_string())] title: String,
    #[prop(into, optional, default = "movie".to_string())] kind: String,
    #[prop(default = None)] items: Option<Vec<MediaItem>>,
) -> impl IntoView {
    let is_tv = kind == "tv";
    let items_prop = items.clone();
    let kind_c = kind.clone();
    let items_res = LocalResource::new(move || {
        let items_opt = items_prop.clone();
        let kind_str = kind_c.clone();
        async move {
            if let Some(it) = items_opt {
                if !it.is_empty() {
                    return it.into_iter().take(10).collect::<Vec<_>>();
                }
            }
            // no items given: the engine's Top 10 for this kind (films for "all"), TMDB trending only when the engine cannot be reached
            let tab = if kind_str == "tv" { "top10_series" } else { "top10_movies" };
            if let Some(rows) = crate::services::ai_engine::fetch_new_popular(Some(tab)).await {
                if let Some(row) = rows.first() {
                    return row.to_media_items().into_iter().take(10).collect::<Vec<_>>();
                }
            }
            let results = fetch_trending(&kind_str).await.unwrap_or_default();
            results.into_iter().take(10).collect::<Vec<_>>()
        }
    });

    let scroll_ref = NodeRef::<leptos::html::Div>::new();
    let (can_scroll_left, set_can_scroll_left) = signal(false);
    let (can_scroll_right, set_can_scroll_right) = signal(true);
    // the first page turn starts the loop: from then on the row can also be paged back from its start, to its end
    let (looping, set_looping) = signal(false);

    let update_scroll_state = Callback::new(move |_: ()| {
        if let Some(el) = scroll_ref.get() {
            let cur = el.scroll_left() as f64;
            let client_w = el.client_width() as f64;
            let scroll_w = el.scroll_width() as f64;
            set_can_scroll_left.set(cur > 10.0);
            set_can_scroll_right.set(scroll_w > client_w + 10.0);
        }
    });

    // Window resize listener
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
        move |direction: &str| {
            if let Some(el) = scroll_ref.get() {
                set_looping.set(true);
                let client_width = el.client_width() as f64;
                let current_scroll = el.scroll_left() as f64;
                let scroll_width = el.scroll_width() as f64;
                let step = client_width * 0.9;
                if crate::components::media::row_scroll::turn_page(&el, direction != "left", crate::utils::layout::app_x()) {
                    let update_c = update.clone();
                    let _ = set_timeout_ms(move || { update_c.run(()); }, 600);
                    return;
                }
                let target = crate::components::media::row_scroll::page_target(direction != "left", current_scroll, client_width, scroll_width, step);
                el.scroll_to_with_x_and_y(target, 0.0);

                let update_c = update.clone();
                let _ = set_timeout_ms(move || {
                    update_c.run(());
                }, 600);
            }
        }
    };

    view! {
        <div class="group relative row-block z-10">
            <h2 class="px-[var(--app-x,56px)] row-title cursor-pointer flex items-center group/title w-fit">
                {title}
                <span class="text-xs text-explore ml-2 opacity-0 group-hover/title:opacity-100 transition-opacity duration-(--motion-slow) flex items-center font-semibold">
                    {crate::i18n::t("row-explore-all")}
                </span>
            </h2>

            <div class="relative group/row" style="container-type: inline-size;">
                <div
                    node_ref=scroll_ref
                    on:scroll=on_scroll
                    // from 600 wide the same columns as every other row (Netflix: slots 8 px apart from the page margin, measured 2026-10-23); phones keep the app's slots
                    // side by side; the extra room below is for the badges
                    class="flex min-[600px]:gap-[var(--tile-gap)] overflow-x-scroll [scrollbar-width:none] [-ms-overflow-style:none] [&::-webkit-scrollbar]:hidden pt-[2px] pb-[42px] -mb-10 w-full items-end pointer-events-auto relative z-10"
                    style="scroll-behavior: smooth;"
                >
                    <div data-loop-spacer="1" class="row-edge" />

                    <Suspense fallback=move || view! { <crate::components::skeleton::TopTenSlotsSkeleton /> }>
                        {move || items_res.get().map(|media| {
                            // the cards, with a copy of the last ones in front (`order: -2`, so it comes first whatever its place in the markup) and a copy of the first ones behind,
                            // so that the row can be paged round (see row_scroll.rs)
                            let n = media.len();
                            let k = if n > 6 { n.min(8) } else { 0 };
                            let leads: Vec<(usize, MediaItem)> = media[n - k..].iter().cloned().enumerate().map(|(i, it)| (n - k + i, it)).collect();
                            let copies: Vec<MediaItem> = media[..k].iter().cloned().collect();
                            let mut views: Vec<AnyView> = media.into_iter().enumerate().map(|(idx, item)| {
                                view! { <TopTenCard item=item index=idx is_tv=is_tv loop_role=if idx == 0 { 1 } else { 0 } /> }.into_any()
                            }).collect();
                            views.extend(copies.into_iter().enumerate().map(|(idx, item)| {
                                view! { <TopTenCard item=item index=idx is_tv=is_tv loop_role=if idx == 0 { 2 } else { 3 } /> }.into_any()
                            }));
                            let lead_views: Vec<AnyView> = leads.into_iter().map(|(idx, item)| view! { <TopTenCard item=item index=idx is_tv=is_tv loop_role=3 /> }.into_any()).collect();
                            views.push(view! { <div data-loop-lead="1" class="flex-none flex min-[600px]:gap-[var(--tile-gap)]" style="display: none; order: -2;">{lead_views}</div> }.into_any());
                            views
                        })}
                    </Suspense>

                    <div class="row-edge" />
                </div>

                <crate::components::media::row_scroll::EdgeButton edge=crate::components::media::row_scroll::Edge::Left visible=Signal::derive(move || can_scroll_left.get() || looping.get()) on_click=Callback::new({ let sl = scroll.clone(); move |_| sl("left") }) label=crate::i18n::t("continue-watching-row-scroll-left") inset="-top-3 bottom-5" />
                <crate::components::media::row_scroll::EdgeButton edge=crate::components::media::row_scroll::Edge::Right visible=can_scroll_right on_click=Callback::new({ let sr = scroll.clone(); move |_| sr("right") }) label=crate::i18n::t("continue-watching-row-scroll-right") inset="-top-3 bottom-5" />
            </div>
        </div>
    }
}
