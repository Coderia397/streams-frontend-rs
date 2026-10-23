//! The page a shared link opens: `/title/tv-1399` or `/title/movie-603` (TMDB gives films and series the same numbers, so the kind is part of the address).
//! A browser that holds an account goes to Home with the title's details window open, as a Netflix member does. Anyone else sees the title's own page, laid out
//! like Netflix's page for people who are not signed in (note 110): the trailer with the sign up form, a summary, trailers, episodes, more details, titles like it,
//! what is trending, and a Get Started button that stays at the bottom once the form has scrolled away. There are no plans or prices: the service is free.

use leptos::prelude::*;
use leptos_router::hooks::{use_navigate, use_params_map};
use crate::components::media::movie_card_badges::RatingBadge;
use crate::components::trailer_player::TrailerPlayer;
use crate::i18n::t;
use crate::services::tmdb::{
    fetch_details, fetch_movie_logo, fetch_recommendations, fetch_season_details, fetch_trending, fetch_videos, select_best_tmdb_trailer, MediaItem, VideoResult,
};

/// `tv-1399` is (true, 1399) and `movie-603` is (false, 603); anything else is not a title.
pub fn parse_slug(slug: &str) -> Option<(bool, u32)> {
    let (kind, id) = slug.split_once('-')?;
    let id = id.parse::<u32>().ok().filter(|i| *i > 0)?;
    match kind {
        "tv" => Some((true, id)),
        "movie" => Some((false, id)),
        _ => None,
    }
}

pub fn slug_for(is_tv: bool, id: u32) -> String {
    format!("{}-{id}", if is_tv { "tv" } else { "movie" })
}

/// The address of a title's page on this site, for sharing.
pub fn share_url(is_tv: bool, id: u32) -> String {
    let origin = web_sys::window().and_then(|w| w.location().origin().ok()).unwrap_or_default();
    format!("{origin}/title/{}", slug_for(is_tv, id))
}

#[component]
pub fn TitlePage() -> impl IntoView {
    let params = use_params_map();
    move || {
        let raw = params.with(|p| p.get("slug").unwrap_or_default());
        match parse_slug(&raw) {
            None => view! { <crate::pages::not_found::NotFoundPage /> }.into_any(),
            // the opener on Home shows the details window once a profile is chosen
            Some(_) if crate::services::account::has_account() => view! { <leptos_router::components::Redirect path=format!("/browse?title={raw}") /> }.into_any(),
            Some((is_tv, id)) => view! { <PublicTitle id=id is_tv=is_tv /> }.into_any(),
        }
    }
}

/// Heading text from a label that ends in a colon ("Cast:" reads "Cast" as a heading), in any language.
fn heading(key: &str) -> String {
    t(key).trim_end().trim_end_matches([':', '：']).trim_end().to_string()
}

fn scroll_to_y(y: f64) {
    if let Some(w) = web_sys::window() {
        let opts = web_sys::ScrollToOptions::new();
        opts.set_top(y);
        opts.set_behavior(web_sys::ScrollBehavior::Smooth);
        w.scroll_to_with_scroll_to_options(&opts);
    }
}

/// Scrolls a section to just under the top of the window.
fn jump_to(id: &str) {
    let Some(w) = web_sys::window() else { return };
    let Some(el) = w.document().and_then(|d| d.get_element_by_id(id)) else { return };
    let top = el.get_bounding_client_rect().top() + w.scroll_y().unwrap_or(0.0) - 16.0;
    scroll_to_y(top.max(0.0));
}

/// The videos for the Trailers row: YouTube trailers, then teasers, then clips, official ones first, at most 10.
fn trailer_list(videos: &[VideoResult]) -> Vec<VideoResult> {
    let rank = |v: &VideoResult| match v.r#type.as_str() {
        "Trailer" => 0,
        "Teaser" => 1,
        _ => 2,
    };
    let mut list: Vec<VideoResult> = videos.iter().filter(|v| v.site == "YouTube" && matches!(v.r#type.as_str(), "Trailer" | "Teaser" | "Clip")).cloned().collect();
    list.sort_by_key(|v| (rank(v), !v.official.unwrap_or(false)));
    list.truncate(10);
    list
}

#[component]
fn PublicTitle(id: u32, is_tv: bool) -> impl IntoView {
    let details = LocalResource::new(move || async move { fetch_details(id, is_tv).await.ok() });
    let logo = LocalResource::new(move || async move { fetch_movie_logo(id, is_tv).await.ok().flatten() });
    let videos = LocalResource::new(move || async move { fetch_videos(id, is_tv).await.unwrap_or_default() });

    let hero_key = RwSignal::new(None::<String>);
    let hero_teaser = RwSignal::new(false);
    let paused = RwSignal::new(false);
    let muted = RwSignal::new(true);
    let ready = RwSignal::new(false);
    let ended = RwSignal::new(false);
    let hero_out = RwSignal::new(false);
    let show_join = RwSignal::new(false);
    let email = RwSignal::new(String::new());
    let hero_ref = NodeRef::<leptos::html::Div>::new();
    let form_ref = NodeRef::<leptos::html::Div>::new();

    if let Some(w) = web_sys::window() {
        w.scroll_to_with_x_and_y(0.0, 0.0);
    }

    // the best trailer plays in the hero, muted, once the list is in (Netflix does the same)
    Effect::new(move |_| {
        let Some(list) = videos.get() else { return };
        if hero_key.get_untracked().is_none() {
            if let Some((key, teaser)) = select_best_tmdb_trailer(&list) {
                hero_teaser.set(teaser);
                hero_key.set(Some(key));
            }
        }
    });

    Effect::new(move |_| {
        let Some(Some(d)) = details.get() else { return };
        if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
            doc.set_title(&crate::i18n::t_args(
                "title-page-doc",
                &[("title", fluent_bundle::FluentValue::from(d.display_title().to_string())), ("brand", fluent_bundle::FluentValue::from(crate::brand::NAME))],
            ));
        }
    });

    // the trailer stops while the hero is off screen, and Get Started appears at the bottom once the form has gone by
    crate::utils::listen::listen_window(leptos::ev::scroll, move |_| {
        let below = |r: &NodeRef<leptos::html::Div>| r.get_untracked().map(|el| el.get_bounding_client_rect().bottom() < 0.0).unwrap_or(false);
        let (h, f) = (below(&hero_ref), below(&form_ref));
        if hero_out.get_untracked() != h {
            hero_out.set(h);
        }
        if show_join.get_untracked() != f {
            show_join.set(f);
        }
    });

    let navigate = use_navigate();
    let go_signup = Callback::new(move |_: ()| {
        navigate(&format!("/signup?email={}", js_sys::encode_uri_component(&email.get_untracked())), Default::default());
    });
    let on_submit = move |e: leptos::ev::SubmitEvent| {
        e.prevent_default();
        go_signup.run(());
    };
    let sign_in = format!("/login?next={}", js_sys::encode_uri_component(&format!("/browse?title={}", slug_for(is_tv, id))));

    let play_video = Callback::new(move |key: String| {
        ended.set(false);
        ready.set(false);
        hero_teaser.set(false);
        hero_key.set(Some(key));
        paused.set(false);
        muted.set(false);
        scroll_to_y(0.0);
    });

    let backdrop = move || {
        details.get().flatten().and_then(|d| d.backdrop_path).map(|p| format!("{}/original{}", crate::config::TMDB_IMAGE_URL, p)).unwrap_or_default()
    };
    let title_text = move || details.get().flatten().map(|d| d.display_title().to_string()).unwrap_or_default();

    view! {
        <div class="min-h-screen bg-void font-sans text-ink">
            <header class="flex items-center justify-between px-4 py-3 md:px-12 md:py-5">
                <a href="/" aria-label=crate::brand::NAME><img src=crate::brand::LOGO_WORDMARK alt=crate::brand::NAME class="h-7 w-auto md:h-9" /></a>
                <a href=sign_in class="rounded-control bg-brand px-4 py-1.5 text-fine font-bold text-ink hover:bg-brand-hover md:px-5 md:py-2 md:text-body-sm">
                    {t("login-sign-in")}
                </a>
            </header>

            // jump links: a strip on phones, a pill over the top of the hero from a tablet up
            <nav aria-label=t("title-jump-label") class="relative z-20 border-b border-ink/10 md:-mb-5 md:flex md:justify-center md:border-0">
                <div class="flex gap-1 overflow-x-auto px-2 [scrollbar-width:none] md:rounded-full md:bg-panel/90 md:px-3 md:shadow-lg md:backdrop-blur [&::-webkit-scrollbar]:hidden">
                    {move || (!trailer_list(&videos.get().unwrap_or_default()).is_empty()).then(|| view! {
                        <button type="button" on:click=move |_| jump_to("title-trailers") class="shrink-0 cursor-pointer px-3 py-3 text-body-sm font-semibold text-ink/90 hover:text-ink md:py-2">{t("title-trailers")}</button>
                    })}
                    {is_tv.then(|| view! {
                        <button type="button" on:click=move |_| jump_to("title-episodes") class="shrink-0 cursor-pointer px-3 py-3 text-body-sm font-semibold text-ink/90 hover:text-ink md:py-2">{t("media-episodes")}</button>
                    })}
                    <button type="button" on:click=move |_| jump_to("title-more-to-watch") class="shrink-0 cursor-pointer px-3 py-3 text-body-sm font-semibold text-ink/90 hover:text-ink md:py-2">{t("title-more-to-watch")}</button>
                </div>
            </nav>

            // hero: the trailer in a rounded panel; the sign up form sits over it from a tablet up and under it on a phone
            <section class="relative px-3 pt-3 md:px-[3.5%] md:pt-0">
                <div node_ref=hero_ref class="relative aspect-video w-full overflow-hidden rounded-card bg-raised [container-type:size] md:aspect-auto md:h-[min(72vh,36rem)] md:rounded-[1.5rem]">
                    <img
                        src=backdrop
                        alt=""
                        class="absolute inset-0 h-full w-full object-cover transition-opacity duration-(--motion-slowest)"
                        class=("opacity-0", move || ready.get() && !ended.get() && !paused.get())
                    />
                    // a 16:9 box that covers the panel at any shape (the panel is wider than the video on a computer and taller on a tablet), so YouTube never adds bars
                    <div class="absolute left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2" style="width: max(100cqw, 177.78cqh); height: max(100cqh, 56.25cqw)">
                        <TrailerPlayer
                            video_key=Signal::derive(move || hero_key.get())
                            is_teaser=Signal::derive(move || hero_teaser.get())
                            is_playing=Signal::derive(move || !paused.get() && !ended.get() && !hero_out.get())
                            is_muted=Signal::derive(move || muted.get())
                            variant="hero".to_string()
                            iframe_id="title-trailer-iframe".to_string()
                            on_ready=Callback::new(move |_| ready.set(true))
                            on_ended=Callback::new(move |_| {
                                ended.set(true);
                                ready.set(false);
                            })
                        />
                    </div>
                    <div class="pointer-events-none absolute inset-0 bg-linear-to-t from-void/70 via-transparent to-transparent md:bg-linear-to-r md:from-void/85 md:via-void/30"></div>

                    {move || hero_key.get().is_some().then(|| view! {
                        <div class="absolute bottom-3 end-3 z-20 flex gap-2 md:bottom-10 md:end-10">
                            {move || (!ended.get()).then(|| view! {
                                <button
                                    type="button"
                                    on:click=move |_| paused.update(|p| *p = !*p)
                                    class="flex h-10 w-10 cursor-pointer items-center justify-center rounded-full border-2 border-ink/60 bg-void/40 text-ink hover:border-ink"
                                    title=move || if paused.get() { t("media-play") } else { t("title-pause") }
                                    aria-label=move || if paused.get() { t("media-play") } else { t("title-pause") }
                                >
                                    <i class=move || if paused.get() { "ph-fill ph-play text-lg" } else { "ph-fill ph-pause text-lg" }></i>
                                </button>
                            })}
                            <crate::components::media::action_buttons::MuteReplayButton
                                is_muted=Signal::derive(move || muted.get())
                                is_ended=Signal::derive(move || ended.get())
                                on_toggle=Callback::new(move |_| {
                                    if ended.get_untracked() {
                                        ended.set(false);
                                        paused.set(false);
                                    } else {
                                        muted.update(|m| *m = !*m);
                                    }
                                })
                                size="md".to_string()
                            />
                        </div>
                    })}
                </div>

                <div node_ref=form_ref class="relative z-10 mx-auto mt-6 w-full max-w-[22rem] space-y-4 px-2 text-center md:absolute md:bottom-10 md:start-[calc(3.5%+2.5rem)] md:mt-0 md:px-0">
                    <img src=crate::brand::LOGO_WORDMARK alt="" class="mx-auto h-5 w-auto md:h-6" />
                    <h1 class="flex min-h-16 items-center justify-center">
                        {move || match logo.get().flatten() {
                            Some(src) => view! { <img src=src alt=title_text class="mx-auto max-h-24 max-w-full object-contain drop-shadow-text md:max-h-28" /> }.into_any(),
                            None => view! { <span class="text-heading-xl font-black leading-tight drop-shadow-text">{title_text}</span> }.into_any(),
                        }}
                    </h1>
                    <form on:submit=on_submit class="space-y-3">
                        <input
                            type="email"
                            autocomplete="email"
                            prop:value=move || email.get()
                            on:input=move |e| email.set(event_target_value(&e))
                            placeholder=t("login-email-address")
                            aria-label=t("auth-email")
                            class="h-12 w-full rounded-control border border-ink/40 bg-void/60 px-4 text-body-md text-ink backdrop-blur-md placeholder:text-ink/60 focus:border-ink focus:outline-none"
                        />
                        <button type="submit" class="mx-auto flex h-12 w-full max-w-[14rem] cursor-pointer items-center justify-center gap-2 rounded-control bg-brand text-body-lg font-bold text-ink transition-colors hover:bg-brand-hover active:scale-95">
                            {t("login-get-started")}
                            <i class="ph-bold ph-caret-right rtl:rotate-180" aria-hidden="true"></i>
                        </button>
                    </form>
                    <p class="text-body-sm text-ink/90">{t("auth-tick-free")}</p>
                </div>
            </section>

            <main class="mx-auto w-full max-w-[1130px] px-4 pb-6 md:px-8">
                <Suspense fallback=|| view! { <div class="mt-10 h-40 animate-pulse rounded-card bg-panel/60"></div> }>
                    {move || details.get().map(|res| match res {
                        Some(d) => view! { <TitleBody details=d id=id is_tv=is_tv videos=videos play_video=play_video /> }.into_any(),
                        None => view! {
                            <p class="mt-10 rounded-card bg-panel/60 p-6 text-center text-body-md">{t("info-modal-error-loading-details")}</p>
                        }.into_any(),
                    })}
                </Suspense>

                <PosterRow id="title-trending" heading=t("media-badge-trending-now") source=PosterSource::Trending exclude=(is_tv, id) />

                <section class="relative mt-12 overflow-hidden rounded-card">
                    <img src=crate::pages::login::wall("wide") alt="" class="absolute inset-0 h-full w-full object-cover opacity-40" />
                    <div class="absolute inset-0 bg-linear-to-r from-void/80 to-void/30"></div>
                    <div class="relative flex flex-col gap-4 p-6 md:flex-row md:items-center md:justify-between md:px-8">
                        <p class="text-title font-bold md:text-heading">{t("login-unlimited-films-series-and-more")}</p>
                        <a href="/" class="w-fit shrink-0 rounded-full border border-ink/40 bg-void/60 px-5 py-2 text-body-sm font-bold hover:bg-void/80">{t("landing-learn-more")}</a>
                    </div>
                </section>
            </main>

            <div class="mt-10">
                <crate::pages::auth::AuthFooter />
            </div>

            // Get Started once the form has scrolled away
            <div
                class="fixed inset-x-0 bottom-0 z-(--z-nav) flex justify-center pb-[max(1rem,env(safe-area-inset-bottom))] transition-all duration-(--motion-base)"
                class=("invisible", move || !show_join.get())
                class=("opacity-0", move || !show_join.get())
                class=("translate-y-4", move || !show_join.get())
            >
                <button
                    type="button"
                    on:click=move |_| go_signup.run(())
                    class="cursor-pointer rounded-full bg-brand px-12 py-3 text-body-lg font-bold text-ink shadow-2xl hover:bg-brand-hover active:scale-95"
                >
                    {t("login-get-started")}
                </button>
            </div>
        </div>
    }
}

/// Everything under the hero that needs the title's details: the summary card, trailers, episodes, more details and titles like it.
#[component]
fn TitleBody(
    details: crate::services::tmdb::DetailedMediaItem,
    id: u32,
    is_tv: bool,
    videos: LocalResource<Vec<VideoResult>>,
    play_video: Callback<String>,
) -> impl IntoView {
    let title = details.display_title().to_string();
    let year = details.release_date.clone().or(details.first_air_date.clone()).map(|d| d.chars().take(4).collect::<String>()).unwrap_or_default();
    let seasons = details.number_of_seasons.unwrap_or(0);
    let genres: Vec<(u32, String)> = details.genres.clone().unwrap_or_default().into_iter().map(|g| (g.id, g.name)).collect();
    let first_genre = genres.first().map(|(_, n)| n.clone());
    let cast: Vec<String> = details.credits.as_ref().map(|c| c.cast.iter().take(10).map(|m| m.name.clone()).collect()).unwrap_or_default();
    let starring = cast.iter().take(3).cloned().collect::<Vec<_>>().join(", ");
    let creators = details.created_by.as_ref().map(|c| c.iter().map(|p| p.name.clone()).collect::<Vec<_>>().join(", ")).unwrap_or_default();
    let vibe = crate::components::media::info_modal::derive_vibe_tags(&genres, &details.overview).iter().map(|t| crate::i18n::mood_label(t)).collect::<Vec<_>>().join(", ");
    let audio = details
        .original_language
        .as_deref()
        .filter(|l| !l.is_empty())
        .map(|l| crate::i18n::t_args("title-original-audio", &[("language", fluent_bundle::FluentValue::from(crate::i18n::language_name(l)))]));
    let genre_names = genres.iter().map(|(_, n)| n.clone()).collect::<Vec<_>>().join(", ");
    let vibe_heading = if is_tv { heading("media-this-show-is") } else { heading("media-this-movie-is") };
    let dot = || view! { <span class="text-ink/50" aria-hidden="true">"•"</span> };

    view! {
        // summary card
        <section class="mt-8 rounded-card border border-ink/10 bg-panel/80 p-5 md:mt-10 md:p-6">
            <h2 class="text-title font-bold md:text-title-lg">{title.clone()}</h2>
            <p class="mt-2 flex flex-wrap items-center gap-x-2 gap-y-1 text-body-sm font-semibold text-ink-soft">
                {(!year.is_empty()).then(|| view! { <span>{year.clone()}</span> })}
                {(seasons > 0).then(|| view! { {dot()}<span>{crate::i18n::t_args("media-seasons", &[("n", fluent_bundle::FluentValue::from(seasons as i64))])}</span> })}
                {dot()}<RatingBadge id=id is_tv=is_tv size="xs".to_string() />
                {first_genre.map(|g| view! { {dot()}<span>{g}</span> })}
            </p>
            <div class="mt-4 grid gap-4 md:grid-cols-2">
                <p class="text-body-md leading-relaxed">{details.overview.clone()}</p>
                <div class="space-y-1 text-body-sm md:border-s md:border-ink/20 md:ps-5">
                    {(!starring.is_empty()).then(|| view! { <p><span class="font-semibold text-ink-ash">{t("title-starring")}" "</span>{starring}</p> })}
                    {(!creators.is_empty()).then(|| view! { <p><span class="font-semibold text-ink-ash">{t("title-creators")}" "</span>{creators}</p> })}
                </div>
            </div>
        </section>

        // trailers, each plays in the hero
        {move || {
            let list = trailer_list(&videos.get().unwrap_or_default());
            (!list.is_empty()).then(|| view! {
                <section id="title-trailers" class="mt-10">
                    <h2 class="mb-3 text-title font-bold md:text-heading">{t("title-trailers")}</h2>
                    <Rail>
                        {list.into_iter().map(|v| {
                            let key = v.key.clone();
                            let label = match v.r#type.as_str() {
                                "Trailer" => t("title-video-trailer"),
                                "Teaser" => t("title-video-teaser"),
                                _ => t("title-video-clip"),
                            };
                            view! {
                                <button type="button" on:click=move |_| play_video.run(key.clone()) class="group w-[72vw] max-w-[20rem] shrink-0 cursor-pointer snap-start text-start sm:w-[20rem]">
                                    <div class="relative aspect-video overflow-hidden rounded-card bg-raised">
                                        <img src=format!("https://i.ytimg.com/vi/{}/hqdefault.jpg", v.key) alt="" loading="lazy" class="h-full w-full object-cover transition-transform duration-(--motion-base) group-hover:scale-105" />
                                        <span class="absolute bottom-3 start-3 flex h-9 w-9 items-center justify-center rounded-full border-2 border-ink bg-void/40"><i class="ph-fill ph-play text-body-md"></i></span>
                                    </div>
                                    <p class="mt-2 text-caption font-bold uppercase tracking-widest text-ink-ash">{label}</p>
                                    <p class="line-clamp-2 text-body-sm font-semibold">{v.name.clone().unwrap_or_default()}</p>
                                </button>
                            }
                        }).collect_view()}
                    </Rail>
                </section>
            })
        }}

        {(is_tv && seasons > 0).then(|| view! { <Episodes id=id seasons=seasons /> })}

        // more details
        <section class="mt-10">
            <h2 class="mb-3 text-title font-bold md:text-heading">{t("title-more-details")}</h2>
            <div class="grid gap-3 md:grid-cols-3">
                <div class="space-y-3 rounded-card border border-ink/10 bg-panel/80 p-5 text-body-sm">
                    {(!genre_names.is_empty()).then(|| view! { <div><h3 class="font-bold">{heading("media-genres")}</h3><p class="text-ink-soft">{genre_names}</p></div> })}
                    <div><h3 class="font-bold">{vibe_heading}</h3><p class="text-ink-soft">{vibe}</p></div>
                </div>
                {audio.map(|a| view! {
                    <div class="rounded-card border border-ink/10 bg-panel/80 p-5 text-body-sm">
                        <h3 class="font-bold">{t("player-audio")}</h3>
                        <p class="text-ink-soft">{a}</p>
                    </div>
                })}
                {(!cast.is_empty()).then(|| view! {
                    <div class="rounded-card border border-ink/10 bg-panel/80 p-5 text-body-sm">
                        <h3 class="font-bold">{heading("media-cast")}</h3>
                        <p class="text-ink-soft">{cast.join(", ")}</p>
                    </div>
                })}
            </div>
        </section>

        <PosterRow id="title-more-to-watch" heading=t("title-you-might-like") source=PosterSource::Like(is_tv, id) exclude=(is_tv, id) />
    }
}

/// The Episodes section: a season picker and a row of episode cards.
#[component]
fn Episodes(id: u32, seasons: u32) -> impl IntoView {
    let season = RwSignal::new(1u32);
    let episodes = LocalResource::new(move || {
        let s = season.get();
        async move { fetch_season_details(id, s).await.map(|d| d.episodes).unwrap_or_default() }
    });
    view! {
        <section id="title-episodes" class="mt-10">
            <div class="mb-3 flex flex-wrap items-center gap-4">
                <h2 class="text-title font-bold md:text-heading">{t("media-episodes")}</h2>
                {(seasons > 1).then(|| view! {
                    <div class="relative">
                        <select
                            aria-label=t("title-season-label")
                            on:change=move |e| { if let Ok(n) = event_target_value(&e).parse::<u32>() { season.set(n); } }
                            class="cursor-pointer appearance-none rounded-full border border-ink/40 bg-void py-1.5 ps-4 pe-10 text-body-sm font-semibold text-ink hover:border-ink"
                        >
                            {(1..=seasons).map(|n| view! {
                                <option value=n.to_string() selected=move || season.get() == n class="bg-paper text-paper-ink">
                                    {crate::i18n::t_args("media-season-n", &[("n", fluent_bundle::FluentValue::from(n as i64))])}
                                </option>
                            }).collect_view()}
                        </select>
                        <i class="ph-fill ph-caret-down pointer-events-none absolute end-3 top-1/2 -translate-y-1/2 text-fine" aria-hidden="true"></i>
                    </div>
                })}
            </div>
            <Suspense fallback=|| view! { <div class="h-64 animate-pulse rounded-card bg-panel/60"></div> }>
                {move || episodes.get().map(|list| view! {
                    <Rail>
                        {list.into_iter().map(|ep| view! {
                            <div class="w-[72vw] max-w-[20rem] shrink-0 snap-start overflow-hidden rounded-card border border-ink/10 bg-panel/80 p-2 sm:w-[20rem]">
                                <div class="relative aspect-video overflow-hidden rounded-control bg-raised">
                                    {ep.still_url("w300").map(|src| view! { <img src=src alt="" loading="lazy" class="h-full w-full object-cover" /> })}
                                    {ep.runtime.filter(|r| *r > 0).map(|r| view! {
                                        <span class="absolute bottom-2 end-2 rounded-tick bg-void/80 px-1.5 py-0.5 text-caption font-semibold">{format!("{r}m")}</span>
                                    })}
                                </div>
                                <div class="px-2 pb-2 pt-3">
                                    <p class="text-body-sm font-bold">{format!("{}. {}", ep.episode_number, ep.name)}</p>
                                    <p class="mt-1 line-clamp-4 text-body-sm text-ink-soft">{ep.overview.clone()}</p>
                                </div>
                            </div>
                        }).collect_view()}
                    </Rail>
                })}
            </Suspense>
        </section>
    }
}

#[derive(Clone, Copy)]
enum PosterSource {
    /// titles like this one: the recommendation service, then TMDB's recommendations
    Like(bool, u32),
    Trending,
}

/// A row of posters, each a link to that title's own page.
#[component]
fn PosterRow(id: &'static str, heading: String, source: PosterSource, exclude: (bool, u32)) -> impl IntoView {
    let items = LocalResource::new(move || async move {
        let list: Vec<MediaItem> = match source {
            PosterSource::Like(tv, mid) => {
                let ai = crate::services::ai_engine::fetch_ai_recommendations(mid, tv).await.unwrap_or_default();
                if ai.is_empty() { fetch_recommendations(mid, tv).await.unwrap_or_default() } else { ai.into_iter().map(|i| i.to_media_item()).collect() }
            }
            PosterSource::Trending => fetch_trending("all").await.unwrap_or_default(),
        };
        let like_kind = match source {
            PosterSource::Like(tv, _) => Some(tv),
            PosterSource::Trending => None,
        };
        list.into_iter()
            .filter(|m| m.poster_path.as_deref().is_some_and(|p| !p.is_empty()) && !m.is_nsfw())
            .map(|m| {
                let tv = if m.media_type.is_some() { m.is_tv() } else { like_kind.unwrap_or_else(|| m.is_tv()) };
                (tv, m)
            })
            .filter(|(tv, m)| (*tv, m.id) != exclude)
            .take(12)
            .collect::<Vec<_>>()
    });
    view! {
        {move || items.get().filter(|l| !l.is_empty()).map(|list| view! {
            <section id=id class="mt-10">
                <h2 class="mb-3 text-title font-bold md:text-heading">{heading.clone()}</h2>
                <Rail>
                    {list.into_iter().map(|(tv, m)| view! {
                        <a href=format!("/title/{}", slug_for(tv, m.id)) class="w-[30vw] max-w-[11.5rem] shrink-0 snap-start sm:w-[11.5rem]">
                            <img
                                src=m.poster_url("w342").unwrap_or_default()
                                alt=m.display_title().to_string()
                                loading="lazy"
                                class="aspect-[2/3] w-full rounded-card bg-raised object-cover transition-transform duration-(--motion-base) hover:scale-[1.03]"
                            />
                        </a>
                    }).collect_view()}
                </Rail>
            </section>
        })}
    }
}

/// A row that scrolls sideways: swipe on a phone, arrows on the sides from a tablet up.
#[component]
fn Rail(children: Children) -> impl IntoView {
    let track = NodeRef::<leptos::html::Div>::new();
    let by = move |forward: bool| {
        if let Some(el) = track.get_untracked() {
            let rtl = web_sys::window().and_then(|w| w.document()).and_then(|d| d.document_element()).is_some_and(|e| e.get_attribute("dir").as_deref() == Some("rtl"));
            let step = el.client_width() as f64 * 0.8;
            el.scroll_by_with_x_and_y(if forward != rtl { step } else { -step }, 0.0);
        }
    };
    let arrow = "absolute top-1/2 z-10 hidden h-14 w-9 -translate-y-1/2 cursor-pointer items-center justify-center rounded-control bg-lift/90 text-ink opacity-0 shadow-lg transition-opacity group-hover/rail:opacity-100 hover:bg-line-strong focus-visible:opacity-100 md:flex";
    view! {
        <div class="group/rail relative">
            <div node_ref=track class="flex snap-x snap-mandatory gap-3 overflow-x-auto scroll-smooth pb-1 [scrollbar-width:none] [&::-webkit-scrollbar]:hidden">
                {children()}
            </div>
            <button type="button" on:click=move |_| by(false) class=format!("{arrow} -start-4") aria-label=t("vibe-row-scroll-left")>
                <i class="ph-bold ph-caret-left text-title rtl:rotate-180"></i>
            </button>
            <button type="button" on:click=move |_| by(true) class=format!("{arrow} -end-4") aria-label=t("vibe-row-scroll-right")>
                <i class="ph-bold ph-caret-right text-title rtl:rotate-180"></i>
            </button>
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slugs_round_trip() {
        assert_eq!(parse_slug("tv-1399"), Some((true, 1399)));
        assert_eq!(parse_slug("movie-603"), Some((false, 603)));
        assert_eq!(parse_slug(&slug_for(true, 42)), Some((true, 42)));
        assert_eq!(parse_slug("person-5"), None);
        assert_eq!(parse_slug("tv-0"), None);
        assert_eq!(parse_slug("tv-abc"), None);
        assert_eq!(parse_slug("1399"), None);
    }
}
