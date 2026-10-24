//! The title page on the phone form, as Netflix's Android app draws it (measured on a Pixel 8, 2026-10-23, note 116): a page over the
//! current one, under the bottom bar, with a back arrow; the trailer in a 16:9 frame at the top (the backdrop until it plays, a line for
//! how far it is, a mute button); the title, its facts, Play, the synopsis, the cast and the creators; My List, Rate and Share; then the
//! Episodes and More Like This tabs. It shows the title open in the UI store, as the website's window does (info_modal.rs), and the
//! address carries it (`?title=`), so Back closes it.
use leptos::prelude::*;
use leptos::html::Div;
use crate::store::{use_ui_store, use_library_store, LibraryEntry};
use crate::services::tmdb::{fetch_details, fetch_videos, fetch_season_details, fetch_recommendations, select_best_tmdb_trailer, DetailedMediaItem, MediaItem};
use crate::components::media::movie_card_badges::RatingBadge;
use crate::components::media::movie_card_rating::{MovieRating, RatingIcon};
use crate::components::trailer_player::TrailerPlayer;
use crate::components::skeleton::Bone;

#[derive(Clone, Copy, PartialEq)]
enum Tab {
    Episodes,
    MoreLikeThis,
}

#[component]
pub fn PhoneTitle() -> impl IntoView {
    let ui_store = use_ui_store();
    let is_open = ui_store.info_modal_open;
    let movie_id = ui_store.info_modal_movie_id;
    let is_tv = ui_store.info_modal_is_tv;
    let scroller = NodeRef::<Div>::new();
    let tab = RwSignal::new(Tab::Episodes);
    let season = RwSignal::new(1u32);
    let (ready, set_ready) = signal(false);
    let (ended, set_ended) = signal(false);
    let (time, set_time) = signal(0.0f64);
    let (duration, set_duration) = signal(0.0f64);
    let paused = RwSignal::new(false);
    let seek = RwSignal::new(None::<f64>);
    // the trailer's frame is at least half in view (it pauses when the page is scrolled past it, and goes on when it comes back)
    let frame = NodeRef::<Div>::new();
    let in_view = RwSignal::new(true);
    Effect::new(move |_| {
        use wasm_bindgen::JsCast;
        let Some(el) = frame.get() else { return };
        let cb = wasm_bindgen::closure::Closure::<dyn Fn(js_sys::Array)>::new(move |entries: js_sys::Array| {
            for e in entries.iter() {
                if let Ok(entry) = e.dyn_into::<web_sys::IntersectionObserverEntry>() {
                    in_view.try_set(entry.intersection_ratio() >= 0.5);
                }
            }
        });
        let init = web_sys::IntersectionObserverInit::new();
        init.set_threshold(&wasm_bindgen::JsValue::from_f64(0.5));
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

    // another title (from More Like This, or opened anew): its own tab, season and trailer, from the top of the page
    Effect::new(move |_| {
        let _ = (movie_id.get(), is_tv.get());
        tab.set(Tab::Episodes);
        season.set(1);
        set_ready.set(false);
        set_ended.set(false);
        set_time.set(0.0);
        set_duration.set(0.0);
        paused.set(false);
        if let Some(el) = scroller.get_untracked() {
            el.set_scroll_top(0);
        }
    });

    // the page behind does not scroll while this one is open
    Effect::new(move |_| {
        let open = is_open.get();
        if let Some(body) = web_sys::window().and_then(|w| w.document()).and_then(|d| d.body()) {
            let _ = body.style().set_property("overflow", if open { "hidden" } else { "" });
        }
    });
    on_cleanup(|| {
        if let Some(body) = web_sys::window().and_then(|w| w.document()).and_then(|d| d.body()) {
            let _ = body.style().set_property("overflow", "");
        }
    });

    let details = LocalResource::new(move || {
        let (id, tv) = (movie_id.get(), is_tv.get());
        async move {
            match id {
                Some(id) => fetch_details(id, tv).await.ok(),
                None => None,
            }
        }
    });
    let videos = LocalResource::new(move || {
        let (id, tv) = (movie_id.get(), is_tv.get());
        async move {
            match id {
                Some(id) => fetch_videos(id, tv).await.unwrap_or_default(),
                None => Vec::new(),
            }
        }
    });
    // the best trailer YouTube has not refused here, chosen again when it refuses the one playing (services/unplayable.rs)
    let video_key = Memo::new(move |_| {
        crate::services::unplayable::track();
        videos.get().and_then(|v| select_best_tmdb_trailer(&v)).map(|(k, _)| k)
    });
    Effect::new(move |previous: Option<Option<String>>| {
        let key = video_key.get();
        if previous.is_some_and(|p| p != key) {
            set_ready.set(false);
            set_ended.set(false);
            set_time.set(0.0);
        }
        key
    });
    let playing = Signal::derive(move || is_open.get() && !ended.get() && !paused.get() && in_view.get());

    let close = move || is_open.set(false);

    view! {
        <Show when=move || is_open.get()>
            // the fade is opacity only: the enter animation kept a transform on this page for good, and a transform makes the page the frame
            // of everything fixed inside it, so the season menu opened scrolled with the page (owner, 2026-10-24)
            <div
                node_ref=scroller
                class="fixed inset-0 z-(--z-info-modal) overflow-y-auto overscroll-contain scrollbar-hide bg-void text-ink animate-fadeIn"
                role="dialog"
                aria-modal="true"
            >
                // the bar with the back arrow stays at the top, black, while the page scrolls under it
                <div class="sticky top-0 z-20 bg-void pt-[env(safe-area-inset-top)]">
                    <div class="flex h-[52px] items-center">
                        <button
                            type="button"
                            on:click=move |_| close()
                            class="ms-2.5 flex h-11 w-11 items-center justify-center text-ink active:scale-95 transition-transform cursor-pointer"
                            aria-label=crate::i18n::t("navbar-mobile-back")
                        >
                            <i class="ph-bold ph-arrow-left text-heading"></i>
                        </button>
                    </div>
                </div>

                // the trailer's frame: the backdrop until the trailer plays, then the trailer, muted unless the viewer turns the sound on
                <div node_ref=frame class="relative aspect-video w-full overflow-hidden bg-void">
                    {move || details.get().flatten().and_then(|d| d.backdrop_path.clone()).map(|p| {
                        let src = format!("{}/w1280{}", crate::config::TMDB_IMAGE_URL, p);
                        view! {
                            <img
                                srcset=crate::utils::images::backdrop_srcset(&src)
                                sizes="100vw"
                                src=src
                                alt=""
                                class="absolute inset-0 h-full w-full object-cover transition-opacity duration-(--motion-slow)"
                                class=("opacity-0", move || ready.get() && playing.get())
                            />
                        }
                    })}
                    {move || video_key.get().map(|_| view! {
                        <TrailerPlayer
                            video_key=Signal::derive(move || video_key.get())
                            is_playing=playing
                            is_muted=ui_store.preview_muted
                            seek_time=Signal::derive(move || seek.get())
                            variant="card".to_string()
                            iframe_id="phone-title-trailer".to_string()
                            on_ready=Callback::new(move |_| {
                                set_ready.set(true);
                                if let Some(id) = movie_id.get_untracked() { crate::store::trailers::mark(id, is_tv.get_untracked()); }
                            })
                            on_ended=Callback::new(move |_| set_ended.set(true))
                            on_unplayable=Callback::new(|_| {})
                            on_time_update=Callback::new(move |t| set_time.set(t))
                            on_duration=Callback::new(move |d| set_duration.set(d))
                        />
                    })}
                    // the app's controls (tap: pause, scrubber, time, sound), as on New & Hot
                    <Show when=move || video_key.get().is_some() && !ended.get()>
                        <crate::components::trailer_controls::TrailerControls
                            time=Signal::derive(move || time.get())
                            duration=Signal::derive(move || duration.get())
                            paused=paused
                            seek=seek
                            on_scrub=Callback::new(move |t| set_time.set(t))
                        />
                    </Show>
                </div>

                <Suspense fallback=|| view! { <PhoneTitleSkeleton /> }>
                    {move || details.get().map(|d| match d {
                        Some(d) => view! { <TitleBody details=d season=season tab=tab /> }.into_any(),
                        None => view! { <p class="px-2.5 py-10 text-center text-ink-body">{crate::i18n::t("info-modal-error-loading-details")}</p> }.into_any(),
                    })}
                </Suspense>
                // room for the bottom bar, which stays over the page
                <div class="h-[calc(96px+env(safe-area-inset-bottom))]"></div>
            </div>
        </Show>
    }
}

/// Everything under the trailer for one title.
#[component]
fn TitleBody(details: DetailedMediaItem, season: RwSignal<u32>, tab: RwSignal<Tab>) -> impl IntoView {
    let library_store = use_library_store();
    let id = details.id;
    let is_tv = details.number_of_seasons.is_some();
    let title = details.display_title().to_string();
    let year = details.release_date.clone().or(details.first_air_date.clone()).map(|d| d.chars().take(4).collect::<String>()).unwrap_or_default();
    let seasons = details.number_of_seasons.unwrap_or(0);
    let length = if is_tv {
        (seasons > 0).then(|| crate::i18n::t_args("media-seasons", &[("n", fluent_bundle::FluentValue::from(seasons as i64))]))
    } else {
        details.runtime.filter(|r| *r > 0).map(|r| format!("{}h {}m", r / 60, r % 60))
    };
    let cast: Vec<String> = details.credits.as_ref().map(|c| c.cast.iter().take(10).map(|m| m.name.clone()).collect()).unwrap_or_default();
    // the people who made it: a series' creators, a film's directors
    let makers: Vec<String> = if is_tv {
        details.created_by.clone().unwrap_or_default().into_iter().map(|p| p.name).take(3).collect()
    } else {
        details.credits.as_ref().map(|c| c.crew.iter().filter(|p| p.job.as_deref() == Some("Director")).map(|p| p.name.clone()).take(3).collect()).unwrap_or_default()
    };
    let cast_open = RwSignal::new(false);

    let watch_store = crate::store::use_watch_store();
    let record = watch_store.get_record(id, is_tv);
    let started = record.as_ref().is_some_and(|r| r.percentage > 0.0);
    let watch_url = match (&record, is_tv) {
        (Some(r), true) if r.season.is_some() && r.episode.is_some() => format!("/watch/tv/{id}?season={}&episode={}", r.season.unwrap_or(1), r.episode.unwrap_or(1)),
        (_, true) => format!("/watch/tv/{id}?season=1&episode=1"),
        (_, false) => format!("/watch/movie/{id}"),
    };

    let in_list = Signal::derive(move || library_store.my_list.get().contains_key(&id));
    let entry = MediaItem {
        id,
        title: Some(title.clone()),
        name: Some(title.clone()),
        overview: details.overview.clone(),
        poster_path: details.poster_path.clone(),
        backdrop_path: details.backdrop_path.clone(),
        vote_average: details.vote_average,
        release_date: details.release_date.clone(),
        first_air_date: details.first_air_date.clone(),
        media_type: Some(if is_tv { "tv" } else { "movie" }.to_string()),
        ..Default::default()
    };
    let toggle_list = move |_| {
        let mut list = library_store.my_list.get();
        if list.remove(&id).is_none() {
            list.insert(id, LibraryEntry { media: entry.clone(), added_at: js_sys::Date::now() as u64 });
        }
        library_store.my_list.set(list);
    };

    let rating = RwSignal::new(crate::store::ratings::get(id, is_tv));
    let rate_open = RwSignal::new(false);
    let rate = move |r: MovieRating| {
        let next = if rating.get_untracked() == Some(r) { None } else { Some(r) };
        rating.set(next);
        crate::store::ratings::set(id, is_tv, next);
        rate_open.set(false);
    };
    let share = crate::components::share::use_share();
    let share_item = crate::components::share::ShareItem::title(is_tv, id, title.clone(), details.poster_path.as_deref());
    // a film has no episodes: More Like This is its tab
    if !is_tv {
        tab.set(Tab::MoreLikeThis);
    }

    view! {
        <div class="px-2.5 pt-0.5">
            <h1 class="text-heading-lg font-bold leading-[1.2] text-ink">{title.clone()}</h1>
            // year, age rating, length, picture quality
            <div class="mt-2 flex items-center gap-2.5 text-body text-ink-body">
                {(!year.is_empty()).then(|| view! { <span>{year}</span> })}
                <RatingBadge id=id is_tv=is_tv size="xs".to_string() />
                {length.map(|l| view! { <span>{l}</span> })}
                <span class="rounded-tick border border-ink/40 px-[5px] text-caption-sm leading-[16px] text-ink/90">"HD"</span>
            </div>

            <a href=watch_url class="btn-pill btn-ink mt-2.5 h-12 w-full text-body-lg">
                <i class="ph-fill ph-play text-2xl"></i>
                <span>{if started { crate::i18n::t("media-resume") } else { crate::i18n::t("media-play") }}</span>
            </a>

            <p class="mt-2 text-body-md leading-[20.7px] text-ink">{details.overview.clone()}</p>

            {(!cast.is_empty()).then(|| {
                let short = cast.iter().take(3).cloned().collect::<Vec<_>>().join(", ");
                let all = cast.join(", ");
                let more = cast.len() > 3;
                view! {
                    <p class="mt-2.5 text-body-sm leading-[17.6px] text-ink-body">
                        {crate::i18n::t("title-starring")}" "
                        {move || if cast_open.get() { all.clone() } else { short.clone() }}
                        {move || (more && !cast_open.get()).then(|| view! {
                            "… "
                            <button type="button" on:click=move |_| cast_open.set(true) class="text-ink-soft cursor-pointer">{crate::i18n::t("title-more")}</button>
                        })}
                    </p>
                }
            })}
            {(!makers.is_empty()).then(|| view! {
                <p class="text-body-sm leading-[17.6px] text-ink-body">
                    {if is_tv { crate::i18n::t("title-creators") } else { crate::i18n::t("title-directors") }}" "{makers.join(", ")}
                </p>
            })}

            // My List, Rate and Share: an icon over its label, 96 px wide and 8 px apart from 14 px in, as in the app
            <div class="mt-4 flex items-start gap-2 ps-1">
                <button type="button" on:click=toggle_list class="flex w-24 flex-col items-center gap-2.5 py-2 text-ink cursor-pointer active:scale-95 transition-transform">
                    <i class=move || if in_list.get() { "ph-bold ph-check text-title-md" } else { "ph-bold ph-plus text-title-md" }></i>
                    <span class="text-label text-ink-body">{crate::i18n::t("my-list")}</span>
                </button>
                <div class="relative">
                    <button type="button" on:click=move |_| rate_open.update(|o| *o = !*o) class="flex w-24 flex-col items-center gap-2.5 py-2 text-ink cursor-pointer active:scale-95 transition-transform">
                        {move || view! { <RatingIcon rating=rating.get() size=22 mask_color="var(--color-void)".to_string() /> }}
                        <span class="text-label text-ink-body">{move || if rating.get().is_some() { crate::i18n::t("rating-rated") } else { crate::i18n::t("action-buttons-rate") }}</span>
                    </button>
                    // the three choices in a bubble over the button
                    <Show when=move || rate_open.get()>
                        <div class="absolute bottom-full left-1/2 z-10 mb-1 flex -translate-x-1/2 items-center gap-1 rounded-button bg-lift px-2 py-1.5 shadow-xl animate-in fade-in zoom-in-95">
                            {[(MovieRating::Dislike, "rating-not-for-me"), (MovieRating::Like, "rating-like"), (MovieRating::Love, "rating-love")].into_iter().map(|(r, key)| view! {
                                <button
                                    type="button"
                                    on:click=move |_| rate(r)
                                    aria-label=crate::i18n::t(key)
                                    class=move || format!("flex h-11 w-11 items-center justify-center rounded-full cursor-pointer {}", if rating.get() == Some(r) { "bg-ink text-void" } else { "text-ink" })
                                >
                                    <RatingIcon rating=Some(r) size=22 mask_color=if rating.get_untracked() == Some(r) { "var(--color-ink)".to_string() } else { "var(--color-lift)".to_string() } />
                                </button>
                            }).collect_view()}
                        </div>
                    </Show>
                </div>
                <button type="button" on:click=move |_| share.open(share_item.clone()) class="flex w-24 flex-col items-center gap-2.5 py-2 text-ink cursor-pointer active:scale-95 transition-transform">
                    <i class="ph-bold ph-share-network text-title-md"></i>
                    <span class="text-label text-ink-body">{crate::i18n::t("share-button")}</span>
                </button>
            </div>
        </div>

        // the tabs: a line across the page, the chosen tab marked by a bar over it
        // Netflix's tabs: boxes 48 px high with 12 px either side of the label, from 8 px in; the bar over the chosen one spans its box
        <div class="mt-8 border-t border-ink/15">
            <div class="flex px-2">
                {is_tv.then(|| view! { <TabButton label=crate::i18n::t("media-episodes") me=Tab::Episodes tab=tab /> })}
                <TabButton label=crate::i18n::t("phone-title-more-like-this") me=Tab::MoreLikeThis tab=tab />
            </div>
        </div>
        {move || if is_tv && tab.get() == Tab::Episodes {
            view! { <Episodes series_id=id seasons=seasons season=season /> }.into_any()
        } else {
            view! { <MoreLikeThis id=id is_tv=is_tv /> }.into_any()
        }}
    }
}

#[component]
fn TabButton(label: String, me: Tab, tab: RwSignal<Tab>) -> impl IntoView {
    view! {
        <button
            type="button"
            on:click=move |_| tab.set(me)
            class=move || format!("relative flex h-12 items-center px-3 text-body-md font-bold cursor-pointer {}", if tab.get() == me { "text-ink" } else { "text-ink-soft" })
        >
            <span class="absolute inset-x-0 top-0 h-1 bg-brand transition-opacity" class=("opacity-0", move || tab.get() != me)></span>
            {label}
        </button>
    }
}

/// The season menu and the season's episodes: a picture with a play mark, the number and name, the length, and the synopsis under them.
#[component]
fn Episodes(series_id: u32, seasons: u32, season: RwSignal<u32>) -> impl IntoView {
    let menu_open = RwSignal::new(false);
    let episodes = LocalResource::new(move || {
        let n = season.get();
        async move { fetch_season_details(series_id, n).await.map(|s| s.episodes).unwrap_or_default() }
    });
    view! {
        <div class="px-2 pt-2">
            {(seasons > 1).then(|| view! {
                // round, as every dropdown here (owner, 2026-10-23)
                <button type="button" on:click=move |_| menu_open.set(true) class="flex h-9 items-center gap-2 rounded-button bg-sheet ps-3.5 pe-3 text-title-sm font-bold text-ink cursor-pointer">
                    {move || crate::i18n::t_args("media-season-n", &[("n", fluent_bundle::FluentValue::from(season.get() as i64))])}
                    <i class="ph-fill ph-caret-down text-sm opacity-80"></i>
                </button>
            })}
            {(seasons == 1).then(|| view! { <p class="py-2 text-body-md font-bold text-ink">{crate::i18n::t_args("media-season-n", &[("n", fluent_bundle::FluentValue::from(1))])}</p> })}
            <Suspense fallback=|| view! { <crate::components::skeleton::EpisodeRowsSkeleton /> }>
                <div class="mt-2">
                    {move || episodes.get().map(|list| list.into_iter().map(|ep| {
                        let href = format!("/watch/tv/{series_id}?season={}&episode={}", season.get_untracked(), ep.episode_number);
                        let still = ep.still_url("w300");
                        view! {
                            // Netflix's row (note 116): the still 130 x 80 px from 10 px in, the title 12 px after it, the synopsis right under the still
                            <a href=href class="block px-0.5 pt-2 pb-6">
                                <div class="flex items-center gap-3">
                                    <div class="relative aspect-[130/80] w-[130px] shrink-0 overflow-hidden rounded-tile bg-raised">
                                        {still.map(|s| view! { <img src=s alt="" loading="lazy" class="h-full w-full object-cover" /> })}
                                        <span class="absolute inset-0 flex items-center justify-center">
                                            <span class="flex h-9 w-9 items-center justify-center rounded-full border border-ink/80 bg-void/40"><i class="ph-fill ph-play text-base text-ink"></i></span>
                                        </span>
                                    </div>
                                    <div class="min-w-0 flex-1">
                                        <p class="line-clamp-5 text-title-sm font-bold leading-tight text-ink">{format!("{}. {}", ep.episode_number, ep.name)}</p>
                                        {ep.runtime.filter(|r| *r > 0).map(|r| view! { <p class="mt-0.5 text-body-md text-ink-body">{format!("{r}m")}</p> })}
                                    </div>
                                </div>
                                {(!ep.overview.is_empty()).then(|| view! { <p class="mt-3 line-clamp-4 text-body-md leading-[19.4px] text-ink-body">{ep.overview.clone()}</p> })}
                            </a>
                        }
                    }).collect_view())}
                </div>
            </Suspense>
        </div>
        <super::season_picker::SeasonPicker open=menu_open seasons=seasons season=season />
    }
}

/// Titles like this one, three posters to a row; a poster opens its title on this page.
#[component]
fn MoreLikeThis(id: u32, is_tv: bool) -> impl IntoView {
    let ui_store = use_ui_store();
    let items = LocalResource::new(move || async move { fetch_recommendations(id, is_tv).await.unwrap_or_default() });
    view! {
        <Suspense fallback=|| view! {
            <div class="grid grid-cols-3 gap-2 px-2.5 pt-3" role="status" aria-busy="true">
                {(0..6).map(|_| view! { <Bone class="aspect-[2/3] rounded-tile" /> }).collect_view()}
            </div>
        }>
            <div class="grid grid-cols-3 gap-2 px-2.5 pt-3">
                {move || items.get().map(|list| list.into_iter().filter(|m| m.poster_path.is_some()).take(18).map(|m| {
                    let poster = format!("{}/w342{}", crate::config::TMDB_IMAGE_URL, m.poster_path.clone().unwrap_or_default());
                    let tv = m.media_type.as_deref().map(|t| t == "tv").unwrap_or(is_tv);
                    let mid = m.id;
                    let name = m.title.clone().or(m.name.clone()).unwrap_or_default();
                    view! {
                        <button
                            type="button"
                            on:click=move |_| { ui_store.info_modal_is_tv.set(tv); ui_store.info_modal_movie_id.set(Some(mid)); }
                            // radius: a poster tile, the rows' tile corners
                            class="aspect-[2/3] overflow-hidden rounded-tile bg-raised cursor-pointer"
                        >
                            <img src=poster alt=name loading="lazy" class="h-full w-full object-cover" />
                        </button>
                    }
                }).collect_view())}
            </div>
        </Suspense>
    }
}

/// The page's parts in their places while the title loads.
#[component]
fn PhoneTitleSkeleton() -> impl IntoView {
    view! {
        <div class="px-2.5 pt-2" role="status" aria-busy="true">
            <Bone class="h-8 w-2/3 rounded-control" />
            <Bone class="mt-2 h-4 w-1/3 rounded-control" />
            <Bone class="mt-3 h-12 w-full rounded-button" />
            <Bone class="mt-3 h-4 w-full rounded-control" />
            <Bone class="mt-2 h-4 w-full rounded-control" />
            <Bone class="mt-2 h-4 w-3/4 rounded-control" />
        </div>
    }
}
