//! Clips on the phone form, as Netflix's Android app has them behind its Clips tab (recorded on a Pixel 8, 2026-10-23 and 2026-10-24,
//! note 116): a vertical pager of full screen clips, one a page; the clip in view plays with sound, a swipe snaps to the next one, a tap
//! pauses it under a large play mark. On the right: sound, My List, Share and the title's round poster with its logo, which opens the
//! title; at the bottom left: the title, its tags and its synopsis (in full for a moment on the first clip only, then one line with
//! "more"), and a line with its length; at its end a clip plays again. Netflix cuts vertical clips; here a clip is a scene from the title
//! that TMDB lists (its trailer when it lists none), from its first second, cropped to fill the screen. The feed runs on through the
//! titles of the profile's Home rows (about 400) in the order of order.rs, ten at a time; what the person does with a clip goes to the engine.
//! The website has no Clips (lib.rs sends a computer Home).
use leptos::prelude::*;
use leptos::html::Div;
use wasm_bindgen::prelude::*;
use crate::services::tmdb::{fetch_videos, select_clip, MediaItem};
use crate::store::{use_library_store, use_ui_store, LibraryEntry};
use crate::components::trailer_player::TrailerPlayer;
use crate::components::media::icons::app_icons::{AppIcon, Glyph};

mod order;

/// How long the synopsis shows in full when a clip comes into view, before it shortens to one line.
const SYNOPSIS_MS: u64 = 5_000;
/// A video shorter than this is no clip (TMDB files 3 second promos as clips); Netflix's shortest seen was 11 s (2026-10-24).
const MIN_CLIP_SECONDS: f64 = 10.0;
/// Nor one longer than this (a film's first 10 minutes); Netflix's longest seen was 58 s.
const MAX_CLIP_SECONDS: f64 = 300.0;

thread_local! {
    /// Videos passed over for their length, kept while the site is open so a clip that comes round again goes straight to the next video
    static PASSED_OVER: std::cell::RefCell<Vec<String>> = const { std::cell::RefCell::new(Vec::new()) };
}

/// Clips added at a time; the next ones are added when the clip in view is BATCH_AHEAD from the end.
const BATCH: usize = 10;
const BATCH_AHEAD: usize = 4;
/// A clip left this soon after it began playing tells the engine "not for me" (the owner's feed suggestion 3).
const SKIP_MS: f64 = 3_000.0;

#[component]
pub fn ClipsPage() -> impl IntoView {
    // the profile's titles for the personal rows (read here: they come from the page's stores)
    let signals = StoredValue::new(crate::services::ai_engine::personal_signals());
    // the clips on the page with their places; titles come once
    let shown = RwSignal::new(Vec::<(usize, MediaItem)>::new());
    let shown_keys = StoredValue::new(std::collections::HashSet::<(bool, u32)>::new());
    let adding = StoredValue::new(false);
    let no_more = StoredValue::new(false);
    // the next BATCH, in the order of order.rs; the engine's rows are read again each time, so they follow what the person has swiped
    // since (the session's taste, services/ai_engine::send_feedback)
    let add_more = move || {
        if adding.get_value() || no_more.get_value() {
            return;
        }
        adding.set_value(true);
        leptos::task::spawn_local(async move {
            use crate::services::ai_engine::{fetch_feed_page, fetch_new_popular_n};
            let trending = fetch_new_popular_n(Some("watching"), Some(40)).await.and_then(|rows| rows.into_iter().next()).map(|r| r.items).unwrap_or_default();
            let rows = fetch_feed_page("home", None, Some(24), &signals.get_value()).await.map(|p| p.rows).unwrap_or_default();
            let next = order::order(&trending, &rows, &shown_keys.get_value());
            let mut batch: Vec<MediaItem> = next.iter().take(BATCH).map(|v| v.to_media_item()).collect();
            if batch.is_empty() && shown_keys.with_value(|k| k.is_empty()) {
                // the engine cannot be reached: TMDB's trending titles
                batch = crate::services::tmdb::fetch_trending("all").await.unwrap_or_default();
                no_more.set_value(true);
            }
            if batch.is_empty() {
                no_more.set_value(true);
            }
            shown_keys.update_value(|k| k.extend(batch.iter().map(|m| (m.is_tv(), m.id))));
            shown.try_update(|s| {
                let n = s.len();
                s.extend(batch.into_iter().enumerate().map(|(i, m)| (n + i, m)));
            });
            adding.set_value(false);
        });
    };
    add_more();
    // the clip in view (its place in the list)
    let current = RwSignal::new(0usize);
    Effect::new(move |_| {
        let c = current.get();
        let n = shown.with(|s| s.len());
        if n > 0 && c + BATCH_AHEAD >= n {
            add_more();
        }
    });
    // Clips play with sound, as Netflix's; a browser allows sound only once the person has touched the page, so before that they start
    // muted. Its own setting: the trailers elsewhere keep theirs
    let touched = web_sys::window()
        .and_then(|w| js_sys::Reflect::get(&w.navigator(), &"userActivation".into()).ok())
        .and_then(|a| js_sys::Reflect::get(&a, &"hasBeenActive".into()).ok())
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let muted = RwSignal::new(!touched);

    // in the page frame for its bottom bar, which stays over the clips (the frame's top bar stays under them, as the app has none here)
    view! {
        <crate::components::layout::Layout>
            <div class="fixed inset-0 z-(--z-dialog) overflow-y-scroll snap-y snap-mandatory overscroll-contain scrollbar-hide bg-void">
                <Show when=move || shown.with(|s| !s.is_empty()) fallback=|| view! { <div class="h-dvh w-full skeleton"></div> }>
                    <For each=move || shown.get() key=|(i, m)| (*i, m.id) children=move |(i, item)| view! { <Clip item=item index=i current=current muted=muted /> } />
                </Show>
            </div>
        </crate::components::layout::Layout>
    }
}

#[component]
fn Clip(item: MediaItem, index: usize, current: RwSignal<usize>, muted: RwSignal<bool>) -> impl IntoView {
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
    // the player is set up when the clip is in view or next, so the next swipe finds it ready
    let near = Memo::new(move |_| { let c = current.get(); index == c || index == c + 1 });
    // its videos are looked up two clips ahead, so a title with none is gone before a swipe reaches it (the owner's feed suggestion 5)
    let ahead = Memo::new(move |_| { let c = current.get(); index >= c && index <= c + 2 });
    let (videos, set_videos) = signal(Vec::new());
    let (found, set_found) = signal(false);
    let looked_up = StoredValue::new(false);
    Effect::new(move |_| {
        if ahead.get() && !looked_up.get_value() {
            looked_up.set_value(true);
            leptos::task::spawn_local(async move {
                if let Ok(v) = fetch_videos(id, is_tv).await {
                    set_videos.set(v);
                    set_found.set(true);
                }
            });
        }
    });
    // a scene from the title when TMDB lists one, else its trailer; one shorter than MIN_CLIP_SECONDS or longer than MAX_CLIP_SECONDS
    // is passed over once the player has told its length
    let passed_over = RwSignal::new(PASSED_OVER.with(|p| p.borrow().clone()));
    let pick = Memo::new(move |_| {
        crate::services::unplayable::track();
        let skip = passed_over.get();
        select_clip(&videos.get().into_iter().filter(|v| !skip.contains(&v.key)).collect::<Vec<_>>())
    });
    let video_key = Memo::new(move |_| pick.get().map(|(k, _)| k));
    let is_scene = Signal::derive(move || pick.get().is_some_and(|(_, scene)| scene));
    // no video that plays here: the clip is left out of the feed
    let no_video = Memo::new(move |_| found.get() && pick.get().is_none());

    let (ready, set_ready) = signal(false);
    let (time, set_time) = signal(0.0f64);
    let (duration, set_duration) = signal(0.0f64);
    // another video: the picture shows until it plays, and its time and length are its own
    Effect::new(move |was: Option<Option<String>>| {
        let key = video_key.get();
        if was.is_some_and(|w| w != key) {
            set_ready.set(false);
            set_time.set(0.0);
            set_duration.set(0.0);
        }
        key
    });
    let on_length = move |d: f64| {
        if !(MIN_CLIP_SECONDS..=MAX_CLIP_SECONDS).contains(&d) {
            if let Some(key) = video_key.get_untracked() {
                PASSED_OVER.with(|p| p.borrow_mut().push(key.clone()));
                passed_over.update(|s| s.push(key));
            }
        } else {
            set_duration.set(d);
        }
    };
    // the part that plays, from its start to where it loops (components/trailer_player.rs)
    let from = move || crate::components::trailer_player::start_second(0.0, is_scene.get(), false) as f64;
    let length = move || (crate::components::trailer_player::played_end(duration.get()) - from()).max(0.0);
    // the clip's length, which stays as it plays (Netflix's, recorded 2026-10-24)
    let shown_length = move || {
        let s = length().round() as u64;
        format!("{}:{:02}", s / 60, s % 60)
    };
    let fraction = move || if length() > 0.0 { ((time.get() - from()) / length()).clamp(0.0, 1.0) } else { 0.0 };

    // the synopsis in full for a moment on the first clip when the page opens, then one line with "more"; the clips after it come in
    // shortened (Netflix's, recorded 2026-10-24); "more" shows it all
    let full = RwSignal::new(index == 0);
    if index == 0 {
        leptos::leptos_dom::helpers::set_timeout(move || { full.try_set(false); }, std::time::Duration::from_millis(SYNOPSIS_MS));
    }
    // a tap on the clip pauses it and shows a large play mark; another tap plays it on (Netflix's). Each clip starts playing
    let paused = RwSignal::new(false);
    Effect::new(move |_| { if !active.get() { paused.set(false); } });
    let tap = move |e: leptos::ev::MouseEvent| {
        // a tap on a button or on the words is theirs
        let on_control = e.target().and_then(|t| t.dyn_into::<web_sys::Element>().ok()).and_then(|el| el.closest("button, [data-clip-words]").ok().flatten()).is_some();
        if !on_control { paused.update(|p| *p = !*p); }
    };

    // what the person does with the clip goes to the engine's session, which the next clips are ordered by (the owner's feed suggestion
    // 3): a clip watched to its end or its title opened counts a little for the title (click), My List much (like), leaving a playing clip
    // within SKIP_MS against it (skip). None marks the title as seen, so Home still offers it
    let tell = move |event: &'static str| {
        leptos::task::spawn_local(async move {
            crate::services::ai_engine::send_feedback(id, Some(if is_tv { "tv" } else { "movie" }), event).await;
        });
    };
    let playing_since = StoredValue::new(None::<f64>);
    Effect::new(move |_| {
        if active.get() && ready.get() {
            if playing_since.get_value().is_none() {
                playing_since.set_value(Some(js_sys::Date::now()));
            }
        } else if !active.get() {
            if let Some(at) = playing_since.get_value() {
                if js_sys::Date::now() - at < SKIP_MS {
                    tell("skip");
                }
                playing_since.set_value(None);
            }
        }
    });
    let watched_through = StoredValue::new(false);
    let on_looped = move |_| {
        if !watched_through.get_value() {
            watched_through.set_value(true);
            tell("click");
        }
    };

    let in_list = Signal::derive(move || library.my_list.get().contains_key(&id));
    let entry = item.clone();
    let toggle_list = move |_| {
        let mut list = library.my_list.get();
        if list.remove(&id).is_none() {
            list.insert(id, LibraryEntry { media: entry.clone(), added_at: js_sys::Date::now() as u64 });
            tell("like");
        }
        library.my_list.set(list);
    };
    let share = crate::components::share::use_share();
    let share_item = crate::components::share::ShareItem::title(is_tv, id, title.clone(), item.poster_path.as_deref());
    let open_title = move |_| {
        tell("click");
        ui_store.modal_initial_time.set(0.0);
        ui_store.modal_current_time.set(0.0);
        ui_store.info_modal_is_tv.set(is_tv);
        ui_store.info_modal_movie_id.set(Some(id));
        ui_store.info_modal_open.set(true);
    };
    let iframe_id = format!("clip-{}-{id}", if is_tv { "tv" } else { "movie" });
    // the title's logo across the bottom of the round poster, as Netflix's (fetched when the clip is near)
    let logo = LocalResource::new(move || async move { if near.get() { crate::services::tmdb::fetch_movie_logo(id, is_tv).await.ok().flatten() } else { None } });

    view! {
        <div node_ref=root on:click=tap class="relative h-dvh w-full snap-start snap-always overflow-hidden bg-void" class=("hidden", move || no_video.get())>
            // the picture until the trailer plays, cut to the screen
            <img src=picture alt="" class="absolute inset-0 h-full w-full object-cover transition-opacity duration-(--motion-slow)" class=("opacity-0", move || ready.get() && (active.get() || paused.get())) />
            // the trailer: a 16:9 frame as tall as the screen, centred, so it fills the screen
            {move || video_key.get().filter(|_| near.get()).map(|_| view! {
                <div class="pointer-events-none absolute left-1/2 top-0 h-full aspect-video -translate-x-1/2">
                    <TrailerPlayer
                        video_key=Signal::derive(move || video_key.get())
                        is_playing=Signal::derive(move || active.get() && !paused.get())
                        is_muted=muted
                        show_paused=true
                        from_start=is_scene
                        looped=true
                        on_ended=Callback::new(on_looped)
                        variant="hero".to_string()
                        iframe_id=iframe_id.clone()
                        on_ready=Callback::new(move |_| set_ready.set(true))
                        on_unplayable=Callback::new(|_| {})
                        on_time_update=Callback::new(move |t| set_time.set(t))
                        on_duration=Callback::new(on_length)
                    />
                </div>
            })}
            // paused: Netflix's large plain play mark in the middle
            <Show when=move || paused.get()>
                <div class="pointer-events-none absolute inset-0 flex items-center justify-center"><AppIcon glyph=Glyph::Play class="text-icon-xl" /></div>
            </Show>
            // shade under the words
            <div class="pointer-events-none absolute inset-x-0 bottom-0 h-1/2 bg-linear-to-t from-void/80 to-transparent"></div>

            // sound, My List, Share and the title's poster, down the right above the bottom bar (Netflix's, from its view tree, note 116):
            // 48 px buttons 64 px apart and an 86 px poster, all centred 60 px from the right; the poster's foot 47 px above the bar
            // (the bar's top is 72 px above the bottom)
            <div class="absolute end-[17px] bottom-[calc(119px+env(safe-area-inset-bottom))] flex flex-col items-center gap-4">
                <button type="button" on:click=move |_| muted.update(|m| *m = !*m) class="btn-circle btn-circle-lg bg-clip-disc"
                    aria-label=move || if muted.get() { crate::i18n::t("media-unmute") } else { crate::i18n::t("media-mute") }>
                    {move || view! { <AppIcon glyph=if muted.get() { Glyph::Muted } else { Glyph::Speaker } class="text-heading" /> }}
                </button>
                <button type="button" on:click=toggle_list class="btn-circle btn-circle-lg bg-clip-disc" aria-label=crate::i18n::t("action-buttons-add-to-my-list")>
                    {move || view! { <AppIcon glyph=if in_list.get() { Glyph::Check } else { Glyph::Plus } class="text-heading" /> }}
                </button>
                <button type="button" on:click=move |_| share.open(share_item.clone()) class="btn-circle btn-circle-lg bg-clip-disc" aria-label=crate::i18n::t("share-button")>
                    <AppIcon glyph=Glyph::Share class="text-heading" />
                </button>
                // the poster in a circle, the title's logo across its bottom edge (66 by 20 px, as Netflix's)
                <button type="button" on:click=open_title class="relative mt-1 h-[86px] w-[86px] cursor-pointer" aria-label=title.clone()>
                    <span class="block h-full w-full overflow-hidden rounded-full bg-raised"><img src=poster alt="" class="h-full w-full object-cover" /></span>
                    {move || logo.get().flatten().map(|src| view! { <img src=src alt="" class="absolute left-1/2 top-full h-[20px] w-[66px] -translate-x-1/2 -translate-y-1/2 object-contain" /> })}
                </button>
            </div>

            // the title, its tags, the synopsis and the length, at the bottom left: 16 px in, to 116 px from the right, the time line
            // 31 px above the bar
            <div data-clip-words="" class="absolute start-4 end-[116px] bottom-[calc(103px+env(safe-area-inset-bottom))]">
                <button type="button" on:click=open_title class="text-start text-title-md font-bold leading-tight text-ink cursor-pointer">{title.clone()}</button>
                {(!tags.is_empty()).then(|| view! { <p class="mt-1 text-fine font-bold text-ink">{tags.clone()}</p> })}
                // in full for a moment, then one line ending in "more", as Netflix's
                {move || if full.get() {
                    view! { <p class="mt-1 text-body-lg leading-[22px] text-ink animate-fadeIn">{synopsis.clone()}</p> }.into_any()
                } else {
                    view! {
                        <p on:click=move |_| full.set(true) class="mt-1 flex min-w-0 text-body-lg leading-[22px] text-ink cursor-pointer">
                            <span class="truncate">{synopsis.clone()}</span>
                            <span class="shrink-0 ps-1 font-bold">{crate::i18n::t("title-more")}</span>
                        </p>
                    }.into_any()
                }}
                <div class="mt-2 flex items-center gap-3">
                    // a thin line with a small knob where the clip is, and its length in grey
                    <div class="relative h-[2px] flex-1 rounded-full bg-ink/30">
                        <div class="absolute inset-y-0 left-0 rounded-full bg-ink" style=move || format!("width:{:.2}%", fraction() * 100.0)></div>
                        <div class="absolute top-1/2 h-1.5 w-1.5 -translate-x-1/2 -translate-y-1/2 rounded-full bg-ink" style=move || format!("left:{:.2}%", fraction() * 100.0)></div>
                    </div>
                    <span class="text-label tabular-nums text-mute">{shown_length}</span>
                </div>
            </div>
        </div>
    }
}
