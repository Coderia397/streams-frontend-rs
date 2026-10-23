//! Search on phones (note 114): the app's search screen, with its own bar (back, field, voice, clear), the recommendations as a list
//! of rows, and Top Results and More like as rows of tiles.
use super::{Found, Recommended};
use crate::components::media::vibe_row::VibeRow;
use crate::services::tmdb::MediaItem;
use crate::store::use_ui_store;
use leptos::html::Input;
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use leptos_router::NavigateOptions;
use wasm_bindgen::{closure::Closure, JsCast, JsValue};

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
        // Netflix's app (note 116): a 140.5 x 79 px picture, the title 10 px after it at 15 sp (17 px at the owner's font scale)
        <div on:click=open class="flex items-center gap-[10px] w-full cursor-pointer active:opacity-80">
            <div class="relative w-[140.5px] flex-none aspect-video overflow-hidden rounded-field bg-card">
                <img src=picture alt="" class="absolute inset-0 w-full h-full object-cover" loading="lazy" decoding="async" draggable="false" />
            </div>
            <span class="flex-1 min-w-0 text-[17px] font-bold leading-tight text-ink line-clamp-2">{title.clone()}</span>
            <button on:click=play aria-label=crate::i18n::t("media-play") class="btn-circle btn-circle-sm">
                <i class="ph-fill ph-play text-ink text-body-sm ms-px"></i>
            </button>
        </div>
    }
}

#[component]
pub fn PhoneSearch(#[prop(into)] q: Signal<String>, results: Found, recommended: Recommended) -> impl IntoView {
    let navigate = use_navigate();
    // the field: kept in step with the address, and the address follows what is typed (after a short pause, replacing the entry so Back leaves the page)
    let (draft, set_draft) = signal(q.get_untracked());
    Effect::new(move |_| {
        let cur = q.get();
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
    let (focused, set_focused) = signal(true);
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

    // "More like" the first result
    let more_like = LocalResource::new(move || {
        let current_q = q.get();
        async move {
            if current_q.trim().is_empty() { return None; }
            let first = match crate::services::ai_engine::search_titles(&current_q).await { Some(v) => v.into_iter().next()?, None => return None };
            let recs = crate::services::ai_engine::fetch_ai_recommendations(first.id, first.is_tv()).await?;
            Some((first.display_title().to_string(), recs.into_iter().map(|r| r.to_media_item()).collect::<Vec<_>>()))
        }
    });

    view! {
            // ── The bar of the phone layout ─────────────────────────────────────
            <div class="fixed top-0 inset-x-0 z-(--z-nav) bg-field pt-[env(safe-area-inset-top)]">
                <div class="flex items-center h-[52px] ps-1 pe-1">
                    // the app: a back arrow while typing, a magnifier once the keyboard is down
                    {move || if focused.get() {
                        view! {
                            <button on:click=on_back.clone() aria-label=crate::i18n::t("navbar-mobile-back") class="w-12 h-12 flex items-center justify-center text-ink flex-none">
                                <i class="ph ph-arrow-left text-heading-md"></i>
                            </button>
                        }.into_any()
                    } else {
                        view! { <span class="w-12 h-12 flex items-center justify-center text-ink/90 flex-none"><i class="ph ph-magnifying-glass text-heading-md"></i></span> }.into_any()
                    }}
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
                        on:focus=move |_| set_focused.set(true)
                        on:blur=move |_| set_focused.set(false)
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

            <div class="pt-[calc(60px+env(safe-area-inset-top))] px-3 pb-28 min-h-screen bg-void">
                // Nothing typed: the recommendations, as a list
                <Show when=move || q.get().trim().is_empty()>
                    <h1 class="text-[20px] leading-[30px] font-bold text-ink mt-1.5 mb-[5px]">{crate::i18n::t("search-recommended")}</h1>
                    <Suspense fallback=move || view! { <crate::components::skeleton::ListSkeleton /> }>
                        {move || recommended.get().map(|items| match items {
                            Some(items) if !items.is_empty() => view! {
                                <div class="flex flex-col gap-2">
                                    {items.into_iter().take(12).map(|it| view! { <SearchListRow item=it /> }).collect::<Vec<_>>()}
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

                // Something typed: Top Results and More like, as rows of the app's tiles
                <Show when=move || !q.get().trim().is_empty()>
                    <Suspense fallback=move || view! { <div class="-mx-3"><crate::components::skeleton::RowsSkeleton rows=2 /></div> }>
                        {move || results.get().map(|media| {
                            if media.is_empty() {
                                return view! {
                                    <div class="flex flex-col items-center justify-center mt-24 text-center px-6">
                                        <div class="text-title-md font-bold text-ink mb-2">{crate::i18n::t("search-no-matches")}</div>
                                        <div class="text-ink/60 text-base">{crate::i18n::t("search-try-another")}</div>
                                    </div>
                                }.into_any();
                            }
                            view! {
                                <div class="-mx-3">
                                    <VibeRow title=crate::i18n::t("search-top-results") items=media />
                                    {move || more_like.get().flatten().map(|(name, recs)| view! {
                                        <div class="mt-6"><VibeRow title=crate::i18n::t_args("search-more-like", &[("title", fluent_bundle::FluentValue::from(name))]) items=recs /></div>
                                    })}
                                </div>
                            }.into_any()
                        })}
                    </Suspense>
                </Show>
            </div>
    }
}
