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
    let signed_in = crate::services::account::signed_in();
    move || {
        let raw = params.with(|p| p.get("slug").unwrap_or_default());
        match parse_slug(&raw) {
            None => view! { <crate::pages::not_found::NotFoundPage /> }.into_any(),
            // the opener on Home shows the details window once a profile is chosen (also when this browser signs in while the page is open)
            Some(_) if signed_in.get() => view! { <leptos_router::components::Redirect path=format!("/browse?title={raw}") /> }.into_any(),
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
    // the blurred copy of the trailer behind the page (computers only: a phone would decode two videos); it follows the main trailer's time
    let wide = crate::form::current().is_desktop();
    let main_time = StoredValue::new(0.0f64);
    let glow_seek = RwSignal::new(None::<f64>);
    let glow_synced_at = StoredValue::new(0.0f64);

    if let Some(w) = web_sys::window() {
        w.scroll_to_with_x_and_y(0.0, 0.0);
    }

    // the best trailer plays in the hero, muted, once the list is in (Netflix does the same); one YouTube refuses here is replaced by the
    // next best (services/unplayable.rs), or by the picture when there is none
    Effect::new(move |_| {
        crate::services::unplayable::track();
        let Some(list) = videos.get() else { return };
        let current = hero_key.get_untracked();
        if current.as_deref().is_none_or(crate::services::unplayable::is_unplayable) {
            let best = select_best_tmdb_trailer(&list);
            hero_teaser.set(best.as_ref().is_some_and(|(_, t)| *t));
            let next = best.map(|(k, _)| k);
            if next != current {
                hero_key.set(next);
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

    // the mark above the title is the title's own network or studio (Netflix over Stranger Things, HBO over Game of Thrones), as Netflix shows its own mark over its shows
    let maker = move || {
        details.get().flatten().and_then(|d| {
            let pick = |list: Option<Vec<crate::services::tmdb::Company>>| list.unwrap_or_default().into_iter().find(|c| c.logo_path.as_deref().is_some_and(|p| !p.is_empty()));
            let company = if is_tv { pick(d.networks.clone()).or_else(|| pick(d.production_companies.clone())) } else { pick(d.production_companies.clone()) };
            company.map(|c| (c.name, format!("{}/w300{}", crate::config::TMDB_IMAGE_URL, c.logo_path.unwrap_or_default())))
        })
    };

    view! {
        <div class="relative min-h-screen overflow-x-clip bg-title-page font-sans text-ink">
            // the trailer's colours round the panel: a blurred copy of the trailer (computers) over the title's backdrop, as Netflix's page
            <div class="hero-glow pointer-events-none absolute inset-x-0 top-0 h-[520px] overflow-hidden md:h-[910px]" aria-hidden="true" inert>
                // blurred and faint: the 780 px picture is as good as the original here
                <img src=move || backdrop().replace("/original/", "/w780/") alt="" class="h-full w-full scale-110 object-cover" />
                {wide.then(|| view! {
                    // a small player (YouTube then sends a small, cheap picture) scaled up to 121 % of the page's width, as Netflix's copy, then blurred
                    <div class="hero-glow-video absolute inset-0">
                        <div class="absolute left-1/2 top-0 aspect-video w-[12.14vw] origin-top -translate-x-1/2 scale-[10]">
                            <TrailerPlayer
                                video_key=Signal::derive(move || hero_key.get())
                                is_teaser=Signal::derive(move || hero_teaser.get())
                                is_playing=Signal::derive(move || !paused.get() && !ended.get() && !hero_out.get())
                                is_muted=Signal::derive(|| true)
                                seek_time=Signal::derive(move || glow_seek.get())
                                on_unplayable=Callback::new(|_| {})
                                variant="hero".to_string()
                                iframe_id="title-glow-iframe".to_string()
                                on_time_update=Callback::new(move |t: f64| {
                                    let m = main_time.get_value();
                                    let now = js_sys::Date::now();
                                    if m > 0.0 && (t - m).abs() > 1.5 && now - glow_synced_at.get_value() > 3000.0 {
                                        glow_synced_at.set_value(now);
                                        glow_seek.set(Some(m));
                                    }
                                })
                            />
                        </div>
                    </div>
                })}
            </div>

            <header class="relative z-10 flex h-14 items-center justify-between px-6 md:h-[72px] md:px-12">
                <a href="/" aria-label=crate::brand::NAME><img src=crate::brand::LOGO_WORDMARK alt=crate::brand::NAME class="h-7 w-auto md:h-9" /></a>
                <a href=sign_in class="btn-ink btn-pill btn-pill-sm">{t("login-sign-in")}</a>
            </header>

            // jump links: a strip across a phone, a pill centred over the hero from a tablet up (Netflix: 41 px, links 14 px, 70 % white)
            <nav aria-label=t("title-jump-label") class="relative z-10 md:flex md:justify-center">
                <div class="flex overflow-x-auto px-4 [scrollbar-width:none] md:rounded-button md:bg-void/70 [&::-webkit-scrollbar]:hidden">
                    {move || (!trailer_list(&videos.get().unwrap_or_default()).is_empty()).then(|| view! {
                        <button type="button" on:click=move |_| jump_to("title-trailers") class=JUMP_LINK>{t("title-trailers")}</button>
                    })}
                    {is_tv.then(|| view! {
                        <button type="button" on:click=move |_| jump_to("title-episodes") class=JUMP_LINK>{t("media-episodes")}</button>
                    })}
                    <button type="button" on:click=move |_| jump_to("title-more-to-watch") class=JUMP_LINK>{t("title-more-to-watch")}</button>
                </div>
            </nav>

            // hero: the trailer in a rounded panel whose bottom edge curves up (16:9 on a phone; Netflix's 2.19:1 from a tablet up, 606 px tall at 1327 wide); the sign up form sits over it from a tablet up and under it on a phone
            <section class="relative mt-2 px-3 md:mt-4 md:px-12">
                <div class="hero-drop">
                    <div node_ref=hero_ref class="hero-panel relative aspect-video w-full overflow-hidden rounded-[10px] [container-type:size] md:aspect-[1327/606] md:rounded-[20px]">
                        <div class="hero-video absolute inset-0">
                            <img
                                srcset=move || crate::utils::images::backdrop_srcset(&backdrop())
                                sizes=crate::utils::images::WIDE_SIZES
                                src=backdrop
                                alt=""
                                class="absolute inset-0 h-full w-full object-cover transition-opacity duration-(--motion-slowest)"
                                class=("opacity-0", move || ready.get() && !ended.get() && !paused.get())
                            />
                            // a 16:9 box that covers the panel at any shape (the panel is wider than the video on a computer), so YouTube never adds bars
                            <div class="absolute left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2" style="width: max(100cqw, 177.78cqh); height: max(100cqh, 56.25cqw)">
                                <TrailerPlayer
                                    video_key=Signal::derive(move || hero_key.get())
                                    is_teaser=Signal::derive(move || hero_teaser.get())
                                    is_playing=Signal::derive(move || !paused.get() && !ended.get() && !hero_out.get())
                                    is_muted=Signal::derive(move || muted.get())
                                    variant="hero".to_string()
                                    iframe_id="title-trailer-iframe".to_string()
                                    on_time_update=Callback::new(move |t: f64| main_time.set_value(t))
                                    on_ready=Callback::new(move |_| ready.set(true))
                                    on_ended=Callback::new(move |_| {
                                        ended.set(true);
                                        ready.set(false);
                                    })
                                    // a refused trailer is replaced by the next best (the effect above), not counted as ended
                                    on_unplayable=Callback::new(move |_| ready.set(false))
                                />
                            </div>
                        </div>
                        <div class="hero-shade pointer-events-none absolute inset-0"></div>

                        {move || hero_key.get().is_some().then(|| view! {
                            <div class="absolute bottom-5 start-3 z-20 flex gap-3 md:bottom-[72px] md:end-14 md:start-auto">
                                {move || (!ended.get()).then(|| view! {
                                    <button
                                        type="button"
                                        on:click=move |_| paused.update(|p| *p = !*p)
                                        class="btn-circle btn-circle-md"
                                        title=move || if paused.get() { t("media-play") } else { t("title-pause") }
                                        aria-label=move || if paused.get() { t("media-play") } else { t("title-pause") }
                                    >
                                        <i class=move || if paused.get() { "ph-fill ph-play" } else { "ph-fill ph-pause" }></i>
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
                </div>

                // Netflix: a 312 px column 56 px inside the panel and 72 px above its bottom (258 px and centred under the panel on a phone)
                <div node_ref=form_ref class="relative z-10 mx-auto mt-5 flex w-full max-w-[258px] flex-col items-center text-center md:absolute md:bottom-[72px] md:start-[104px] md:mt-0 md:max-w-[312px]">
                    {move || maker().map(|(name, src)| view! { <img src=src alt=name class="h-5 w-auto max-w-[120px] object-contain brightness-0 invert" /> })}
                    <h1 class="mt-3 flex w-full items-center justify-center md:mt-6">
                        {move || match logo.get().flatten() {
                            Some(src) => view! { <img src=src alt=title_text class="max-h-[60px] max-w-[180px] object-contain drop-shadow-text md:max-h-[100px] md:max-w-full" /> }.into_any(),
                            None => view! { <span class="text-heading-lg font-black leading-tight drop-shadow-text md:text-display">{title_text}</span> }.into_any(),
                        }}
                    </h1>
                    <form on:submit=on_submit class="mt-3 flex w-full flex-col items-center md:mt-8">
                        <input
                            type="email"
                            autocomplete="email"
                            prop:value=move || email.get()
                            on:input=move |e| email.set(event_target_value(&e))
                            placeholder=t("login-email-address")
                            aria-label=t("auth-email")
                            class="h-10 w-full border border-scrim-edge bg-scrim-field px-4 text-body-md text-ink placeholder:text-ink/70 focus:border-ink focus:outline-none md:h-14"
                        />
                        <button type="submit" class="btn-brand btn-pill btn-pill-lg mt-2 h-10 w-[200px] md:mt-3 md:h-12 md:w-60 md:text-title-sm">
                            {t("login-get-started")}
                        </button>
                    </form>
                    <p class="mt-2 text-body-sm md:text-body-md">{t("title-free-line")}</p>
                </div>
            </section>

            // Netflix: content 1119 px wide, 24 px from the edges of a phone
            <main class="mx-auto w-full max-w-[1167px] px-6 pb-6">
                <Suspense fallback=|| view! { <crate::components::skeleton::Bone class="mt-8 h-40 rounded-2xl md:mt-[52px]" /> }>
                    {move || details.get().map(|res| match res {
                        Some(d) => view! { <TitleBody details=d id=id is_tv=is_tv videos=videos play_video=play_video /> }.into_any(),
                        None => view! {
                            <p class=format!("{CARD} mt-8 p-6 text-center text-body-md md:mt-[52px]")>{t("info-modal-error-loading-details")}</p>
                        }.into_any(),
                    })}
                </Suspense>

                <PosterRow id="title-trending" heading=t("media-badge-trending-now") source=PosterSource::Trending exclude=(is_tv, id) />

                <section class="relative mt-8 overflow-hidden rounded-2xl md:mt-10">
                    <img src=crate::pages::landing::wall("wide") alt="" class="absolute inset-0 h-full w-full object-cover opacity-40" />
                    <div class="absolute inset-0 bg-linear-to-r from-void/80 to-void/30"></div>
                    <div class="relative flex flex-col gap-4 p-6 md:flex-row md:items-center md:justify-between md:px-6">
                        <p class="text-title font-bold md:text-heading">{t("login-unlimited-films-series-and-more")}</p>
                        <a href="/" class="btn-glass btn-pill btn-pill-sm w-fit shrink-0">{t("landing-learn-more")}</a>
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
                <button type="button" on:click=move |_| go_signup.run(()) class="btn-brand btn-pill btn-pill-lg w-60 text-title-sm shadow-2xl">
                    {t("login-get-started")}
                </button>
            </div>
        </div>
    }
}

/// The page's cards (summary, episodes, More Details): the shared `.surface-card` in style/components.css.
const CARD: &str = "surface-card";
/// A jump link: 14 px, 70 % white, 41 px tall.
const JUMP_LINK: &str = "shrink-0 cursor-pointer px-5 pb-3 pt-[11.2px] text-body-sm font-medium leading-[17.8px] text-ink/70 hover:text-ink";
/// A section heading: 20 px on a phone, 32 px from a tablet up, the line 1.25 of the size (Netflix Sans' normal line).
const SECTION_HEADING: &str = "mb-2.5 text-title font-bold leading-[1.25] md:mb-3.5 md:text-heading-xl";

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
    // each dot travels with the item after it, so a wrapped line never ends on a dot
    let dot = || view! { <span class="text-ink/50" aria-hidden="true">"•"</span> };
    let detail = "text-body-sm font-medium";

    view! {
        // summary card
        <section class=format!("{CARD} mt-8 p-4 md:mt-[52px] md:px-8 md:py-6")>
            <h2 class="text-title font-medium leading-[1.25] md:text-heading">{title.clone()}</h2>
            <p class="mt-3 flex flex-wrap items-center gap-x-2 gap-y-1 text-body-sm font-medium leading-[21px]">
                {(!year.is_empty()).then(|| view! { <span>{year.clone()}</span> })}
                {(seasons > 0).then(|| view! { <span class="inline-flex items-center gap-2">{dot()}<span>{crate::i18n::t_args("media-seasons", &[("n", fluent_bundle::FluentValue::from(seasons as i64))])}</span></span> })}
                <span class="inline-flex items-center gap-2">{dot()}<RatingBadge id=id is_tv=is_tv size="xs".to_string() /></span>
                {first_genre.map(|g| view! { <span class="inline-flex items-center gap-2">{dot()}<span>{g}</span></span> })}
            </p>
            <div class="mt-3 md:grid md:grid-cols-[1fr_1px_1fr] md:gap-x-4">
                <p class="text-body-md leading-normal">{details.overview.clone()}</p>
                <div class="hidden bg-ink/20 md:block" aria-hidden="true"></div>
                <div class="mt-4 space-y-1 text-body-sm leading-[21px] md:mt-0">
                    {(!starring.is_empty()).then(|| view! { <p><span class="font-medium">{t("title-starring")}" "</span>{starring}</p> })}
                    {(!creators.is_empty()).then(|| view! { <p><span class="font-medium">{t("title-creators")}" "</span>{creators}</p> })}
                </div>
            </div>
        </section>

        // trailers, each plays in the hero
        {move || {
            let list = trailer_list(&videos.get().unwrap_or_default());
            (!list.is_empty()).then(|| view! {
                <section id="title-trailers" class="mt-8 md:mt-10">
                    <h2 class=SECTION_HEADING>{t("title-trailers")}</h2>
                    <Rail>
                        {list.into_iter().map(|v| {
                            let key = v.key.clone();
                            let label = match v.r#type.as_str() {
                                "Trailer" => t("title-video-trailer"),
                                "Teaser" => t("title-video-teaser"),
                                _ => t("title-video-clip"),
                            };
                            view! {
                                <button type="button" on:click=move |_| play_video.run(key.clone()) class="group w-[225px] shrink-0 cursor-pointer snap-start text-start md:w-[327px]">
                                    <div class="relative aspect-video overflow-hidden rounded-lg bg-ink/10">
                                        <img src=format!("https://i.ytimg.com/vi/{}/hqdefault.jpg", v.key) alt="" loading="lazy" class="h-full w-full object-cover transition-transform duration-(--motion-base) group-hover:scale-105" />
                                        <span class="btn-circle btn-circle-md absolute bottom-3 start-3"><i class="ph-fill ph-play"></i></span>
                                    </div>
                                    <p class="mt-2 text-label font-bold uppercase leading-none tracking-[0.16em] text-ink/70">{label}</p>
                                    <p class="mt-[3px] line-clamp-2 text-body-md font-medium leading-normal">{v.name.clone().unwrap_or_default()}</p>
                                </button>
                            }
                        }).collect_view()}
                    </Rail>
                </section>
            })
        }}

        {(is_tv && seasons > 0).then(|| view! { <Episodes id=id seasons=seasons /> })}

        // more details
        <section class="mt-8 md:mt-10">
            <h2 class=SECTION_HEADING>{t("title-more-details")}</h2>
            <div class="grid gap-3 md:grid-cols-3">
                <div class=format!("{CARD} flex flex-col gap-2.5 px-4 py-8 md:px-6")>
                    {(!genre_names.is_empty()).then(|| view! { <div class=detail><h3>{heading("media-genres")}</h3><p class="mt-0.5 text-ink/70">{genre_names}</p></div> })}
                    <div class=detail><h3>{vibe_heading}</h3><p class="mt-0.5 text-ink/70">{vibe}</p></div>
                </div>
                {audio.map(|a| view! {
                    <div class=format!("{CARD} flex flex-col gap-2.5 px-4 py-8 md:px-6")>
                        <div class=detail><h3>{t("player-audio")}</h3><p class="mt-0.5 text-ink/70">{a}</p></div>
                    </div>
                })}
                {(!cast.is_empty()).then(|| view! {
                    <div class=format!("{CARD} flex flex-col gap-2.5 px-4 py-8 md:px-6")>
                        <div class=detail><h3>{heading("media-cast")}</h3><p class="mt-0.5 text-ink/70">{cast.join(", ")}</p></div>
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
        <section id="title-episodes" class="mt-8 md:mt-10">
            <div class="mb-2.5 flex flex-wrap items-center gap-4 md:mb-3.5">
                <h2 class="text-title font-bold leading-[1.25] md:text-heading-xl">{t("media-episodes")}</h2>
                {(seasons > 1).then(|| view! {
                    <div class="relative">
                        <select
                            aria-label=t("title-season-label")
                            on:change=move |e| { if let Ok(n) = event_target_value(&e).parse::<u32>() { season.set(n); } }
                            class="h-10 cursor-pointer appearance-none border border-scrim-edge bg-scrim-field ps-3 pe-9 text-body-md text-ink hover:border-ink"
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
            <Suspense fallback=|| view! { <crate::components::skeleton::Bone class="h-[337px] rounded-2xl md:h-[367px]" /> }>
                {move || episodes.get().map(|list| view! {
                    <Rail>
                        {list.into_iter().map(|ep| view! {
                            <div class="surface-card w-[246px] shrink-0 snap-start overflow-hidden p-2 max-md:rounded-xl md:w-[327px]">
                                <div class="relative aspect-video overflow-hidden rounded-xl bg-field-deep md:rounded-2xl">
                                    {ep.still_url("w300").map(|src| view! { <img src=src alt="" loading="lazy" class="h-full w-full object-cover" /> })}
                                    {ep.runtime.filter(|r| *r > 0).map(|r| view! {
                                        <span class="absolute bottom-[9px] end-5 rounded-tick bg-void/70 px-1.5 py-0.5 text-label font-medium text-ink/70">{format!("{r}m")}</span>
                                    })}
                                </div>
                                <div class="px-4 pb-4 pt-[18px]">
                                    <p class="text-body-md font-medium leading-6">{format!("{}. {}", ep.episode_number, ep.name)}</p>
                                    <p class="mt-2 line-clamp-4 text-body-sm leading-[21px] text-ink/70">{ep.overview.clone()}</p>
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

/// A row of posters, each a link to that title's own page (Netflix: 184 x 258 on a computer, 132 x 185 on a phone, 16 px corners).
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
            <section id=id class="mt-8 md:mt-10">
                <h2 class=SECTION_HEADING>{heading.clone()}</h2>
                <Rail inset=true>
                    {list.into_iter().map(|(tv, m)| view! {
                        <a href=format!("/title/{}", slug_for(tv, m.id)) class="w-[132px] shrink-0 snap-start md:w-[184px]">
                            <img
                                src=m.poster_url("w342").unwrap_or_default()
                                alt=m.display_title().to_string()
                                loading="lazy"
                                class="aspect-[184/258] w-full rounded-2xl bg-popover object-cover transition-transform duration-(--motion-base) hover:scale-[1.03]"
                            />
                        </a>
                    }).collect_view()}
                </Rail>
            </section>
        })}
    }
}

/// A row that scrolls sideways: swipe on a phone; Netflix's grey 24 x 80 arrows at the ends, each shown while there is more that way
/// (from a computer up they sit 8 px outside the row). `inset` starts the row 8 px in, as Netflix's poster rows.
#[component]
fn Rail(children: Children, #[prop(optional)] inset: bool) -> impl IntoView {
    let track = NodeRef::<leptos::html::Div>::new();
    let can_back = RwSignal::new(false);
    let can_fwd = RwSignal::new(false);
    let rtl = move || web_sys::window().and_then(|w| w.document()).and_then(|d| d.document_element()).is_some_and(|e| e.get_attribute("dir").as_deref() == Some("rtl"));
    let update = move || {
        if let Some(el) = track.get_untracked() {
            // scrollLeft counts down from 0 in a right to left page
            let left = (el.scroll_left() as f64).abs();
            let room = (el.scroll_width() - el.client_width()) as f64;
            let (b, f) = (left > 2.0, left < room - 2.0);
            if can_back.get_untracked() != b { can_back.set(b); }
            if can_fwd.get_untracked() != f { can_fwd.set(f); }
        }
    };
    // measured once the row is laid out (a frame after it is placed), and again when the window changes size
    Effect::new(move |_| {
        if track.get().is_some() {
            request_animation_frame(update);
        }
    });
    crate::utils::listen::listen_window(leptos::ev::resize, move |_| update());
    let by = move |forward: bool| {
        if let Some(el) = track.get_untracked() {
            let step = el.client_width() as f64 * 0.8;
            el.scroll_by_with_x_and_y(if forward != rtl() { step } else { -step }, 0.0);
        }
    };
    let arrow = "absolute top-1/2 z-10 flex h-20 w-6 -translate-y-1/2 cursor-pointer items-center justify-center rounded-full bg-glass text-ink transition-colors hover:bg-glass-hover";
    view! {
        <div class="relative">
            <div node_ref=track on:scroll=move |_| update() class=format!("flex snap-x snap-mandatory gap-3 overflow-x-auto scroll-smooth [scrollbar-width:none] [&::-webkit-scrollbar]:hidden {}", if inset { "ps-2 scroll-ps-2" } else { "" })>
                {children()}
            </div>
            <Show when=move || can_back.get()>
                <button type="button" on:click=move |_| by(false) class=format!("{arrow} start-1 lg:-start-8") aria-label=t("vibe-row-scroll-left")>
                    <i class="ph-bold ph-caret-left text-body-md rtl:rotate-180"></i>
                </button>
            </Show>
            <Show when=move || can_fwd.get()>
                <button type="button" on:click=move |_| by(true) class=format!("{arrow} end-1 lg:-end-8") aria-label=t("vibe-row-scroll-right")>
                    <i class="ph-bold ph-caret-right text-body-md rtl:rotate-180"></i>
                </button>
            </Show>
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
