//! The Search page. On a phone it is the app's search screen: a bar at the top (back arrow, field, microphone or clear), "Recommended series & films" as a list of rows before
//! anything is typed, and "Top Results" with a row "More like <title>" after. From 640 px the bar of the top navigation is the field and the results are a grid.
use leptos::prelude::*;
use leptos::html::Input;
use leptos_router::hooks::{use_navigate, use_query_map};
use leptos_router::NavigateOptions;
use wasm_bindgen::{closure::Closure, JsCast, JsValue};
use crate::components::layout::Layout;
use crate::components::media::movie_card::MovieCard;
use crate::components::media::vibe_row::VibeRow;
use crate::services::tmdb::{search_media, MediaItem};
use crate::store::use_ui_store;

/// Starts the browser's speech recognition (when it has one) and hands the spoken text to `on_text`. Returns false when the browser cannot listen.
fn listen(on_text: impl Fn(String) + 'static) -> bool {
    let Some(win) = web_sys::window() else { return false };
    let ctor = ["SpeechRecognition", "webkitSpeechRecognition"].iter().find_map(|n| js_sys::Reflect::get(&win, &JsValue::from_str(n)).ok().filter(|c| c.is_function()));
    let Some(ctor) = ctor else { return false };
    let Ok(func) = ctor.dyn_into::<js_sys::Function>() else { return false };
    let Ok(rec) = js_sys::Reflect::construct(&func, &js_sys::Array::new()) else { return false };
    if let Some(lang) = win.navigator().language() {
        let _ = js_sys::Reflect::set(&rec, &JsValue::from_str("lang"), &JsValue::from_str(&lang));
    }
    let cb = Closure::<dyn Fn(JsValue)>::wrap(Box::new(move |ev: JsValue| {
        let text = js_sys::Reflect::get(&ev, &JsValue::from_str("results")).ok()
            .and_then(|r| js_sys::Reflect::get(&r, &JsValue::from_f64(0.0)).ok())
            .and_then(|r| js_sys::Reflect::get(&r, &JsValue::from_f64(0.0)).ok())
            .and_then(|a| js_sys::Reflect::get(&a, &JsValue::from_str("transcript")).ok())
            .and_then(|t| t.as_string());
        if let Some(t) = text { on_text(t); }
    }));
    let _ = js_sys::Reflect::set(&rec, &JsValue::from_str("onresult"), cb.as_ref());
    cb.forget();
    js_sys::Reflect::get(&rec, &JsValue::from_str("start")).ok().and_then(|f| f.dyn_into::<js_sys::Function>().ok()).is_some_and(|f| f.call0(&rec).is_ok())
}

fn can_listen() -> bool {
    web_sys::window().is_some_and(|w| ["SpeechRecognition", "webkitSpeechRecognition"].iter().any(|n| js_sys::Reflect::get(&w, &JsValue::from_str(n)).is_ok_and(|c| c.is_function())))
}

/// One row of the list on the phone: the 16:9 picture, the title, and a round play button.
#[component]
fn SearchListRow(item: MediaItem) -> impl IntoView {
    let ui_store = use_ui_store();
    let navigate = use_navigate();
    let (id, is_tv) = (item.id, item.is_tv());
    let title = item.display_title().to_string();
    let picture = item.backdrop_url("w300").or_else(|| item.poster_url("w300")).unwrap_or_default();
    let open = move |_| {
        ui_store.hero_paused_by_modal.set(false);
        ui_store.modal_initial_time.set(0.0);
        ui_store.modal_current_time.set(0.0);
        ui_store.info_modal_movie_id.set(Some(id));
        ui_store.info_modal_is_tv.set(is_tv);
        ui_store.info_modal_open.set(true);
    };
    let play = move |e: leptos::ev::MouseEvent| {
        e.stop_propagation();
        navigate(&format!("/watch/{}/{}", if is_tv { "tv" } else { "movie" }, id), Default::default());
    };
    view! {
        <div on:click=open class="flex items-center gap-3 w-full cursor-pointer active:opacity-80">
            <div class="relative w-[33%] flex-none aspect-video overflow-hidden rounded-field bg-card">
                <img src=picture alt="" class="absolute inset-0 w-full h-full object-cover" loading="lazy" decoding="async" draggable="false" />
            </div>
            <span class="flex-1 min-w-0 text-body-md font-bold leading-tight text-ink line-clamp-2">{title.clone()}</span>
            <button on:click=play aria-label=crate::i18n::t("media-play") class="btn-circle btn-circle-sm">
                <i class="ph-fill ph-play text-ink text-body-sm ms-px"></i>
            </button>
        </div>
    }
}

#[component]
pub fn SearchPage() -> impl IntoView {
    let query_map = use_query_map();
    let navigate = use_navigate();
    let q = move || query_map.read().get("q").unwrap_or_default();

    // the field: kept in step with the address, and the address follows what is typed (after a short pause, replacing the entry so Back leaves the page)
    let (draft, set_draft) = signal(q());
    Effect::new(move |_| {
        let cur = q();
        if cur != draft.get_untracked() { set_draft.set(cur); }
    });
    let timer = StoredValue::new_local(None::<i32>);
    let nav_for_input = navigate.clone();
    let go = move |text: String| {
        let nav = nav_for_input.clone();
        if let Some(win) = web_sys::window() {
            if let Some(t) = timer.get_value() { win.clear_timeout_with_handle(t); }
            let cb = Closure::once_into_js(move || {
                let encoded: String = js_sys::encode_uri_component(text.trim()).into();
                let url = if encoded.is_empty() { "/search".to_string() } else { format!("/search?q={encoded}") };
                nav(&url, NavigateOptions { replace: true, ..Default::default() });
            });
            let handle = win.set_timeout_with_callback_and_timeout_and_arguments_0(cb.unchecked_ref(), 280).ok();
            timer.set_value(handle);
        }
    };
    let go_input = go.clone();
    let on_input = move |ev: leptos::ev::Event| {
        let v = event_target_value(&ev);
        set_draft.set(v.clone());
        go_input(v);
    };
    let input_ref = NodeRef::<Input>::new();
    Effect::new(move |_| {
        if let Some(i) = input_ref.get() { let _ = i.focus(); }
    });
    let go_voice = go.clone();
    let on_voice = move |_| {
        let go_v = go_voice.clone();
        listen(move |t| { set_draft.set(t.clone()); go_v(t); });
    };
    let go_clear = go.clone();
    let on_clear = move |_| { set_draft.set(String::new()); go_clear(String::new()); if let Some(i) = input_ref.get() { let _ = i.focus(); } };
    let nav_back = navigate.clone();
    let on_back = move |_| {
        let can = web_sys::window().and_then(|w| w.history().ok()).and_then(|h| h.length().ok()).is_some_and(|n| n > 1);
        if can { if let Some(h) = web_sys::window().and_then(|w| w.history().ok()) { let _ = h.back(); } } else { nav_back("/browse", Default::default()); }
    };

    // what is shown: the search, or the recommendations before anything is typed
    let results = LocalResource::new(move || {
        let current_q = q();
        async move {
            if current_q.trim().is_empty() {
                vec![]
            } else {
                match crate::services::ai_engine::search_titles(&current_q).await {
                    Some(items) => items,
                    None => search_media(&current_q).await.unwrap_or_default(),
                }
            }
        }
    });
    // before anything is typed: the engine's suggestions for this session, with the Home feed's picks row as the fallback
    let recommended = LocalResource::new(move || {
        let signals = crate::services::ai_engine::personal_signals();
        async move {
            if let Some(items) = crate::services::ai_engine::fetch_search_suggestions(12).await.filter(|v| !v.is_empty()) {
                return Some(items);
            }
            let page = crate::services::ai_engine::fetch_feed_page("home", None, Some(6), &signals).await?;
            let row = page.rows.into_iter().find(|r| r.title_key.as_deref() == Some("row.todays_top_picks"))?;
            Some(row.to_media_items())
        }
    });
    // "More like" the first result
    let more_like = LocalResource::new(move || {
        let current_q = q();
        async move {
            if current_q.trim().is_empty() { return None; }
            let first = match crate::services::ai_engine::search_titles(&current_q).await { Some(v) => v.into_iter().next()?, None => return None };
            let recs = crate::services::ai_engine::fetch_ai_recommendations(first.id, first.is_tv()).await?;
            Some((first.display_title().to_string(), recs.into_iter().map(|r| r.to_media_item()).collect::<Vec<_>>()))
        }
    });

    view! {
        <Layout>
            // ── The bar of the phone layout ─────────────────────────────────────
            <div class="sm:hidden fixed top-0 inset-x-0 z-(--z-nav) bg-field pt-[env(safe-area-inset-top)]">
                <div class="flex items-center h-[52px] ps-1 pe-1">
                    <button on:click=on_back aria-label=crate::i18n::t("navbar-mobile-back") class="w-12 h-12 flex items-center justify-center text-ink flex-none">
                        <i class="ph ph-arrow-left text-heading-md"></i>
                    </button>
                    <input
                        node_ref=input_ref
                        type="text"
                        inputmode="search"
                        enterkeyhint="search"
                        autocomplete="off"
                        autocapitalize="off"
                        spellcheck="false"
                        placeholder=crate::i18n::t("search-placeholder")
                        aria-label=crate::i18n::t("search-bar-search-input")
                        prop:value=move || draft.get()
                        on:input=on_input.clone()
                        class="flex-1 min-w-0 bg-transparent text-title-md text-ink placeholder:text-ink/55 outline-none border-0 ps-3"
                    />
                    {move || if draft.get().is_empty() {
                        can_listen().then(|| view! {
                            <button on:click=on_voice.clone() aria-label=crate::i18n::t("search-voice") class="w-12 h-12 flex items-center justify-center text-ink/90 flex-none">
                                <i class="ph ph-microphone text-heading-md"></i>
                            </button>
                        }).into_any()
                    } else {
                        view! {
                            <button on:click=on_clear.clone() aria-label=crate::i18n::t("search-bar-clear-search") class="w-12 h-12 flex items-center justify-center text-ink/90 flex-none">
                                <i class="ph ph-x text-heading-md"></i>
                            </button>
                        }.into_any().into()
                    }}
                </div>
            </div>

            <div class="pt-[calc(60px+env(safe-area-inset-top))] sm:pt-28 px-3 sm:px-[var(--app-x,56px)] pb-28 sm:pb-12 min-h-screen bg-void sm:bg-page">
                // Nothing typed: the recommendations
                <Show when=move || q().trim().is_empty()>
                    <h1 class="text-title-md font-bold text-ink mb-3 mt-1 ps-1">{crate::i18n::t("search-recommended")}</h1>
                    <Suspense fallback=move || view! { <div class="space-y-3 animate-pulse">{(0..6).map(|_| view! { <div class="h-[84px] bg-card rounded-field"></div> }).collect::<Vec<_>>()}</div> }>
                        {move || recommended.get().map(|items| match items {
                            Some(items) if !items.is_empty() => view! {
                                <div class="sm:hidden flex flex-col gap-3">
                                    {items.iter().take(12).cloned().map(|it| view! { <SearchListRow item=it /> }).collect::<Vec<_>>()}
                                </div>
                                <div class="hidden sm:grid grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 gap-x-2 gap-y-5">
                                    {items.into_iter().take(15).map(|item| tile(item)).collect::<Vec<_>>()}
                                </div>
                            }.into_any(),
                            _ => view! {
                                <div class="flex flex-col items-center justify-center mt-20 text-center">
                                    <div class="text-xl text-ink mb-2">{crate::i18n::t("search-explore")}</div>
                                    <div class="text-ink/60 text-sm">{crate::i18n::t("search-explore-hint")}</div>
                                </div>
                            }.into_any(),
                        })}
                    </Suspense>
                </Show>

                // Something typed: Top Results and More like
                <Show when=move || !q().trim().is_empty()>
                    <Suspense fallback=move || view! { <div class="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 gap-2.5 animate-pulse">{(0..10).map(|_| view! { <div class="aspect-video bg-card rounded-control"></div> }).collect::<Vec<_>>()}</div> }>
                        {move || results.get().map(|media| {
                            if media.is_empty() {
                                return view! {
                                    <div class="flex flex-col items-center justify-center mt-24 text-center px-6">
                                        <div class="text-title-md font-bold text-ink mb-2">{crate::i18n::t("search-no-matches")}</div>
                                        <div class="text-ink/60 text-base">{crate::i18n::t("search-try-another")}</div>
                                    </div>
                                }.into_any();
                            }
                            let for_row = media.clone();
                            view! {
                                // phone: rows with the tiles of the app
                                <div class="sm:hidden -mx-3">
                                    <VibeRow title=crate::i18n::t("search-top-results") items=for_row />
                                    {move || more_like.get().flatten().map(|(name, recs)| view! {
                                        <div class="mt-6"><VibeRow title=crate::i18n::t_args("search-more-like", &[("title", fluent_bundle::FluentValue::from(name))]) items=recs /></div>
                                    })}
                                </div>
                                // wider: the grid, under Netflix's "More to explore" line (16 px; the titles found that carry the words searched, each a search of
                                // its own, 8 px either side of a thin bar; the grid 24 px below, measured 2026-10-23)
                                <div class="hidden sm:block">
                                    <h1 class="sr-only">{crate::i18n::t_args("search-results-for", &[("query", fluent_bundle::FluentValue::from(q()))])}</h1>
                                    {
                                        let related = more_to_explore(&media, &q());
                                        (!related.is_empty()).then(|| view! {
                                            <div class="mb-6 flex items-start gap-2 text-body-md leading-6">
                                                <span class="shrink-0 text-ink/50">{crate::i18n::t("search-more-to-explore")}</span>
                                                <ul class="flex flex-wrap gap-y-3">
                                                    {related.into_iter().map(|name| {
                                                        let href = format!("/search?q={}", String::from(js_sys::encode_uri_component(&name)));
                                                        view! { <li class="me-2 border-e border-ink/50 pe-2 last:me-0 last:border-e-0 last:pe-0"><a href=href class="text-ink hover:underline">{name}</a></li> }
                                                    }).collect::<Vec<_>>()}
                                                </ul>
                                            </div>
                                        })
                                    }
                                    <div class="grid grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 gap-x-2 gap-y-5">
                                        {media.into_iter().map(|item| tile(item)).collect::<Vec<_>>()}
                                    </div>
                                </div>
                            }.into_any()
                        })}
                    </Suspense>
                </Show>
            </div>
        </Layout>
    }
}

/// The titles found that carry every word searched, once each, at most ten: Netflix's "More to explore" line.
fn more_to_explore(media: &[MediaItem], query: &str) -> Vec<String> {
    let words: Vec<String> = query.split_whitespace().map(str::to_lowercase).collect();
    if words.is_empty() {
        return Vec::new();
    }
    let mut seen = std::collections::HashSet::new();
    media
        .iter()
        .map(|m| m.display_title().trim().to_string())
        .filter(|name| !name.is_empty() && words.iter().all(|w| name.to_lowercase().contains(w.as_str())))
        .filter(|name| seen.insert(name.to_lowercase()))
        .take(10)
        .collect()
}

/// A tile of the grid (wider screens).
fn tile(item: MediaItem) -> impl IntoView {
    let is_tv = item.is_tv();
    let title = item.display_title().to_string();
    let poster = item.poster_url("w342").unwrap_or_default();
    // a title without a wide picture shows its poster (cropped to the tile) instead of a broken image
    let backdrop = item.backdrop_url("w780").or_else(|| item.poster_url("w500")).unwrap_or_default();
    view! {
        <MovieCard
            movie_id=item.id
            is_tv=is_tv
            title=title
            backdrop_path=backdrop
            poster_path=poster
            vote_average=item.vote_average
            overview=item.overview.clone()
            genre_ids=item.genre_ids.clone()
            vibe_pills=item.vibe_pills.clone()
            is_new=item.badges.iter().any(|b| b == "new")
            is_grid=true
        />
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn titled(name: &str) -> MediaItem {
        MediaItem { title: Some(name.to_string()), ..Default::default() }
    }

    #[test]
    fn more_to_explore_lists_titles_with_every_word_once() {
        let found = [titled("The Office"), titled("Office Space"), titled("Suits"), titled("the office"), titled("Office Christmas Party")];
        assert_eq!(more_to_explore(&found, "office"), vec!["The Office", "Office Space", "Office Christmas Party"]);
        assert_eq!(more_to_explore(&found, "office party"), vec!["Office Christmas Party"]);
        assert!(more_to_explore(&found, "  ").is_empty());
    }
}
