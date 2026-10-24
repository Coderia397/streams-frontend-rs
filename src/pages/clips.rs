//! Clips on the phone form, as Netflix's Android app has them behind its Clips tab (recorded on a Pixel 8, 2026-10-23, note 116): a
//! vertical pager of full screen clips, one a page; the clip in view plays, a swipe snaps to the next one, and the sound setting carries
//! over. On the right: sound, My List, Share and the title's round poster, which opens the title; at the bottom left: the title, its
//! tags and its synopsis (shortened to one line with "more" after a few seconds), and a line with the time left. Netflix cuts vertical
//! clips; here a clip is the title's trailer, cropped to fill the screen. The website has no Clips (lib.rs sends a computer Home).
use leptos::prelude::*;
use leptos::html::Div;
use wasm_bindgen::prelude::*;
use crate::services::tmdb::{fetch_videos, select_best_tmdb_trailer, MediaItem};
use crate::store::{use_library_store, use_ui_store, LibraryEntry};
use crate::components::trailer_player::TrailerPlayer;
use crate::components::media::icons::app_icons::{AppIcon, Glyph};

/// How long the synopsis shows in full when a clip comes into view, before it shortens to one line.
const SYNOPSIS_MS: u64 = 5_000;

#[component]
pub fn ClipsPage() -> impl IntoView {
    let feed = LocalResource::new(move || async move {
        // what is being watched today, from the engine (age limit applied there); TMDB only when the engine cannot be reached
        if let Some(rows) = crate::services::ai_engine::fetch_new_popular_n(Some("watching"), Some(20)).await {
            if let Some(row) = rows.first() {
                return row.to_media_items();
            }
        }
        crate::services::tmdb::fetch_trending("all").await.unwrap_or_default()
    });
    // the clip in view (its place in the list)
    let current = RwSignal::new(0usize);

    // in the page frame for its bottom bar, which stays over the clips (the frame's top bar stays under them, as the app has none here)
    view! {
        <crate::components::layout::Layout>
            <div class="fixed inset-0 z-(--z-dialog) overflow-y-scroll snap-y snap-mandatory overscroll-contain scrollbar-hide bg-void">
                <Suspense fallback=|| view! { <div class="h-dvh w-full skeleton"></div> }>
                    {move || feed.get().map(|items| items.into_iter().enumerate().map(|(i, item)| view! { <Clip item=item index=i current=current /> }).collect_view())}
                </Suspense>
            </div>
        </crate::components::layout::Layout>
    }
}

#[component]
fn Clip(item: MediaItem, index: usize, current: RwSignal<usize>) -> impl IntoView {
    let ui_store = use_ui_store();
    let library = use_library_store();
    let root = NodeRef::<Div>::new();
    let id = item.id;
    let is_tv = item.is_tv();
    let title = item.display_title().to_string();
    let picture = item.backdrop_url("w1280").or_else(|| item.poster_url("w780")).unwrap_or_default();
    let poster = item.poster_url("w185").unwrap_or_default();
    let tags = item.vibe_pills.iter().take(4).cloned().collect::<Vec<_>>().join("  •  ");
    let synopsis = item.overview.clone();

    // in view: this clip plays and the page counts it as the current one
    Effect::new(move |_| {
        let Some(el) = root.get() else { return };
        let cb = Closure::<dyn Fn(js_sys::Array)>::wrap(Box::new(move |entries: js_sys::Array| {
            for e in entries.iter() {
                if let Ok(entry) = e.dyn_into::<web_sys::IntersectionObserverEntry>() {
                    if entry.is_intersecting() && entry.intersection_ratio() >= 0.6 {
                        current.try_set(index);
                    }
                }
            }
        }));
        let init = web_sys::IntersectionObserverInit::new();
        init.set_threshold(&JsValue::from_f64(0.6));
        if let Ok(obs) = web_sys::IntersectionObserver::new_with_options(cb.as_ref().unchecked_ref(), &init) {
            obs.observe(&el);
            let kept = StoredValue::new_local(Some((obs, cb)));
            on_cleanup(move || {
                kept.try_update_value(|k| {
                    if let Some((obs, _)) = k.take() {
                        obs.disconnect();
                    }
                });
            });
        }
    });
    let active = Memo::new(move |_| current.get() == index);
    // the trailer is looked up when the clip is in view or next, so the next swipe finds it ready
    let near = Memo::new(move |_| { let c = current.get(); index == c || index == c + 1 });
    let (videos, set_videos) = signal(Vec::new());
    let looked_up = StoredValue::new(false);
    Effect::new(move |_| {
        if near.get() && !looked_up.get_value() {
            looked_up.set_value(true);
            leptos::task::spawn_local(async move {
                if let Ok(v) = fetch_videos(id, is_tv).await {
                    set_videos.set(v);
                }
            });
        }
    });
    let video_key = Memo::new(move |_| {
        crate::services::unplayable::track();
        select_best_tmdb_trailer(&videos.get()).map(|(k, _)| k)
    });

    let (ready, set_ready) = signal(false);
    let (time, set_time) = signal(0.0f64);
    let (duration, set_duration) = signal(0.0f64);
    let left = move || {
        let s = (duration.get() - time.get()).max(0.0).round() as u64;
        format!("{}:{:02}", s / 60, s % 60)
    };
    let fraction = move || if duration.get() > 0.0 { (time.get() / duration.get()).clamp(0.0, 1.0) } else { 0.0 };

    // the synopsis in full for a moment each time the clip comes into view, then one line with "more"
    let full = RwSignal::new(true);
    Effect::new(move |_| {
        if active.get() {
            full.set(true);
            leptos::leptos_dom::helpers::set_timeout(move || { full.try_set(false); }, std::time::Duration::from_millis(SYNOPSIS_MS));
        }
    });

    let in_list = Signal::derive(move || library.my_list.get().contains_key(&id));
    let entry = item.clone();
    let toggle_list = move |_| {
        let mut list = library.my_list.get();
        if list.remove(&id).is_none() {
            list.insert(id, LibraryEntry { media: entry.clone(), added_at: js_sys::Date::now() as u64 });
        }
        library.my_list.set(list);
    };
    let share = crate::components::share::use_share();
    let share_item = crate::components::share::ShareItem::title(is_tv, id, title.clone(), item.poster_path.as_deref());
    let open_title = move |_| {
        ui_store.modal_initial_time.set(0.0);
        ui_store.modal_current_time.set(0.0);
        ui_store.info_modal_is_tv.set(is_tv);
        ui_store.info_modal_movie_id.set(Some(id));
        ui_store.info_modal_open.set(true);
    };
    let iframe_id = format!("clip-{}-{id}", if is_tv { "tv" } else { "movie" });

    view! {
        <div node_ref=root class="relative h-dvh w-full snap-start snap-always overflow-hidden bg-void">
            // the picture until the trailer plays, cut to the screen
            <img src=picture alt="" class="absolute inset-0 h-full w-full object-cover transition-opacity duration-(--motion-slow)" class=("opacity-0", move || ready.get() && active.get()) />
            // the trailer: a 16:9 frame as tall as the screen, centred, so it fills the screen
            {move || video_key.get().filter(|_| near.get()).map(|_| view! {
                <div class="pointer-events-none absolute left-1/2 top-0 h-full aspect-video -translate-x-1/2">
                    <TrailerPlayer
                        video_key=Signal::derive(move || video_key.get())
                        is_playing=Signal::derive(move || active.get())
                        is_muted=ui_store.preview_muted
                        variant="hero".to_string()
                        iframe_id=iframe_id.clone()
                        on_ready=Callback::new(move |_| set_ready.set(true))
                        on_ended=Callback::new(move |_| set_time.set(0.0))
                        on_unplayable=Callback::new(|_| {})
                        on_time_update=Callback::new(move |t| set_time.set(t))
                        on_duration=Callback::new(move |d| set_duration.set(d))
                    />
                </div>
            })}
            // shade under the words
            <div class="pointer-events-none absolute inset-x-0 bottom-0 h-1/2 bg-linear-to-t from-void/80 to-transparent"></div>

            // sound, My List, Share and the title's poster, down the right above the bottom bar
            <div class="absolute end-2 bottom-[calc(132px+env(safe-area-inset-bottom))] flex flex-col items-center gap-4">
                <button type="button" on:click=move |_| ui_store.set_preview_muted(!ui_store.preview_muted.get_untracked()) class="btn-circle btn-circle-md bg-void/40"
                    aria-label=move || if ui_store.preview_muted.get() { crate::i18n::t("media-unmute") } else { crate::i18n::t("media-mute") }>
                    {move || view! { <AppIcon glyph=if ui_store.preview_muted.get() { Glyph::Muted } else { Glyph::Speaker } class="text-xl" /> }}
                </button>
                <button type="button" on:click=toggle_list class="btn-circle btn-circle-md bg-void/40" aria-label=crate::i18n::t("action-buttons-add-to-my-list")>
                    {move || view! { <AppIcon glyph=if in_list.get() { Glyph::Check } else { Glyph::Plus } class="text-xl" /> }}
                </button>
                <button type="button" on:click=move |_| share.open(share_item.clone()) class="btn-circle btn-circle-md bg-void/40" aria-label=crate::i18n::t("share-button")>
                    <AppIcon glyph=Glyph::Share class="text-xl" />
                </button>
                <button type="button" on:click=open_title class="h-16 w-16 overflow-hidden rounded-full border-2 border-ink/80 bg-raised cursor-pointer" aria-label=title.clone()>
                    <img src=poster alt="" class="h-full w-full object-cover" />
                </button>
            </div>

            // the title, its tags, the synopsis and the time left, at the bottom left
            <div class="absolute start-4 end-24 bottom-[calc(96px+env(safe-area-inset-bottom))]">
                <button type="button" on:click=open_title class="text-start text-title-md font-bold leading-tight text-ink cursor-pointer">{title.clone()}</button>
                {(!tags.is_empty()).then(|| view! { <p class="mt-1 text-label font-medium text-ink">{tags.clone()}</p> })}
                <p
                    on:click=move |_| full.set(true)
                    class=move || format!("mt-1 text-body-sm leading-[18px] text-ink {}", if full.get() { "line-clamp-4" } else { "line-clamp-1" })
                >
                    {synopsis.clone()}
                </p>
                <div class="mt-2 flex items-center gap-3">
                    <div class="relative h-[3px] flex-1 rounded-full bg-ink/30">
                        <div class="absolute inset-y-0 left-0 rounded-full bg-ink" style=move || format!("width:{:.2}%", fraction() * 100.0)></div>
                    </div>
                    <span class="text-label tabular-nums text-ink-body">{left}</span>
                </div>
            </div>
        </div>
    }
}
