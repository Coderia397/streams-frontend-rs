use wasm_bindgen::JsCast;
use leptos::prelude::*;
use crate::components::layout::Layout;
use crate::components::media::row::Row;
use crate::components::media::top_ten_row::TopTenRow;
use crate::components::media::spotlight_card::SpotlightCard;
use crate::components::layout::subnav::new_popular_phone::{NewPopularSubNavMobile, NewPopularTab};
use crate::services::tmdb::{fetch_trending, fetch_top_rated, fetch_row_content};

/// The titles of one tab of the phone layout: the engine's row for the tab, TMDB when the engine cannot be reached.
async fn tab_items(tab: NewPopularTab) -> Vec<crate::services::tmdb::MediaItem> {
    let engine_tab = match tab {
        NewPopularTab::Watching => "watching",
        NewPopularTab::Top10Movies => "top10_movies",
        NewPopularTab::Top10Series => "top10_series",
        NewPopularTab::JustLanded => "just_landed",
        NewPopularTab::Games => "",
        NewPopularTab::ComingSoon => "coming_soon",
    };
    if let Some(rows) = crate::services::ai_engine::fetch_new_popular(Some(engine_tab)).await {
        if let Some(row) = rows.first() {
            return row.to_media_items();
        }
    }
    match tab {
        NewPopularTab::Watching => fetch_trending("all").await.unwrap_or_default(),
        NewPopularTab::Top10Movies => fetch_top_rated("movie").await.unwrap_or_default(),
        NewPopularTab::Top10Series => fetch_top_rated("tv").await.unwrap_or_default(),
        NewPopularTab::Games => Vec::new(),
        NewPopularTab::JustLanded => fetch_row_content("movie", None, None, None, Some("/movie/now_playing"), 1).await.unwrap_or_default(),
        NewPopularTab::ComingSoon => fetch_row_content("movie", None, None, None, Some("/movie/upcoming"), 1).await.unwrap_or_default(),
    }
}

fn section_id(tab: NewPopularTab) -> &'static str {
    match tab {
        NewPopularTab::ComingSoon => "nh-coming-soon",
        NewPopularTab::Watching => "nh-watching",
        NewPopularTab::JustLanded => "nh-just-landed",
        NewPopularTab::Games => "nh-games",
        NewPopularTab::Top10Series => "nh-top10-series",
        NewPopularTab::Top10Movies => "nh-top10-films",
    }
}

/// The tabs in the order of the page; one long page that runs from one to the next, as in the app. The pills at the top jump to a tab and follow the scrolling.
const SECTIONS: [NewPopularTab; 5] = [NewPopularTab::ComingSoon, NewPopularTab::Watching, NewPopularTab::Games, NewPopularTab::Top10Series, NewPopularTab::Top10Movies];

/// The Games tab: the games the engine offers, one card each (nothing while Games is off).
#[component]
fn NewHotGames() -> impl IntoView {
    let games = LocalResource::new(|| async { crate::services::ai_engine::fetch_games().await });
    view! {
        <section id="nh-games" class="space-y-6 scroll-mt-[150px]">
            {move || games.get().unwrap_or_default().into_iter().map(|g| view! {
                // radius: a game card
                <a href=g.url.clone() target="_blank" rel="noopener" class="block bg-void border border-ink/25 rounded-pill overflow-hidden">
                    <div class="aspect-video bg-card"><img src=g.image.clone() alt="" class="w-full h-full object-cover" loading="lazy" /></div>
                    <div class="px-5 pb-6 pt-4 flex flex-col gap-2">
                        <h3 class="text-ink font-bold text-title leading-tight">{g.title.clone()}</h3>
                        <p class="text-body-lg text-ink-soft line-clamp-3">{g.description.clone()}</p>
                        <span class="text-ink/60 text-body">{g.genre.clone()}</span>
                    </div>
                </a>
            }).collect::<Vec<_>>()}
        </section>
    }
}

/// One tab of the phone layout: its cards, one under the other.
#[component]
fn NewHotSection(tab: NewPopularTab) -> impl IntoView {
    let items = LocalResource::new(move || async move { tab_items(tab).await });
    view! {
        <section id=section_id(tab) class="space-y-6 scroll-mt-[150px]">
            <Suspense fallback=move || view! {
                <div class="space-y-4 animate-pulse">
                    {(0..2).map(|_| view! { <div class="w-full aspect-video bg-card rounded-modal"></div> }).collect::<Vec<_>>()}
                </div>
            }>
                {move || items.get().map(|items| {
                    let is_top10 = tab == NewPopularTab::Top10Movies || tab == NewPopularTab::Top10Series;
                    let is_coming = tab == NewPopularTab::ComingSoon;
                    items.into_iter().enumerate().take(10).map(|(idx, m)| {
                        let backdrop = m.backdrop_url("w780").unwrap_or_default();
                        let release_date = if is_coming { m.release_date.clone().or_else(|| m.first_air_date.clone()) } else { None };
                        view! {
                            <SpotlightCard
                                movie_id=m.id
                                is_tv=m.is_tv()
                                title=m.display_title().to_string()
                                backdrop_path=backdrop
                                overview=m.overview.clone()
                                rank=is_top10.then_some(idx + 1)
                                is_coming_soon=is_coming
                                release_date=release_date
                            />
                        }
                    }).collect::<Vec<_>>()
                })}
            </Suspense>
        </section>
    }
}

/// The bottom edge of the row of pills (it sticks under the header): where the first card of a tab comes to rest.
fn pills_bottom(doc: &web_sys::Document) -> f64 {
    doc.get_element_by_id("nh-pills").map(|e| e.get_bounding_client_rect().bottom()).unwrap_or(150.0)
}

/// The cards slide into darkness as they leave and come out of it as they arrive: each card is shaded in step with how far its middle is from the middle of the screen (up to about two thirds dark),
/// by the variable `--dim` that the shade of the card reads.
fn dim_cards(doc: &web_sys::Document) {
    let Some(height) = doc.document_element().map(|e| e.client_height() as f64).filter(|h| *h > 0.0) else { return };
    let Ok(cards) = doc.query_selector_all(".spot-card") else { return };
    for i in 0..cards.length() {
        let Some(el) = cards.item(i).and_then(|n| n.dyn_into::<web_sys::HtmlElement>().ok()) else { continue };
        let rect = el.get_bounding_client_rect();
        let away = ((rect.top() + rect.height() / 2.0 - height / 2.0).abs() / (height * 0.75)).max(0.0);
        let dim = ((away - 0.2) / 0.6).clamp(0.0, 1.0) * 0.65;
        let _ = el.style().set_property("--dim", &format!("{dim:.3}"));
    }
}

#[component]
pub fn NewPopularPage() -> impl IntoView {
    let (active_tab, set_active_tab) = signal(NewPopularTab::ComingSoon);

    // the pill follows the scrolling: the last tab whose top has reached the pills is the one in use
    crate::utils::listen::listen_window(leptos::ev::scroll, move |_| {
        let Some(doc) = web_sys::window().and_then(|w| w.document()) else { return };
        let mut current = SECTIONS[0];
        for tab in SECTIONS {
            if let Some(el) = doc.get_element_by_id(section_id(tab)) {
                if el.get_bounding_client_rect().top() <= pills_bottom(&doc) + 24.0 { current = tab; }
            }
        }
        if active_tab.get_untracked() != current { set_active_tab.set(current); }
        dim_cards(&doc);
    });
    // the cards load one after the other: the shades are set again while they come in (and on every scroll, above)
    if let Ok(handle) = set_interval_with_handle(move || { if let Some(doc) = web_sys::window().and_then(|w| w.document()) { dim_cards(&doc); } }, std::time::Duration::from_millis(400)) {
        on_cleanup(move || handle.clear());
    }
    let ui_store = crate::store::use_ui_store();
    let jump_to = Callback::new(move |tab: NewPopularTab| {
        set_active_tab.set(tab);
        let Some(win) = web_sys::window() else { return };
        let Some(doc) = win.document() else { return };
        if let Some(el) = doc.get_element_by_id(section_id(tab)) {
            // the first card of the tab comes to rest right under the pills, so the card before it is out of sight
            let top = (el.get_bounding_client_rect().top() + win.scroll_y().unwrap_or(0.0) - pills_bottom(&doc)).max(0.0);
            let distance = (top - win.scroll_y().unwrap_or(0.0)).abs();
            // already at the top of the tab (or a few pixels off): nothing moves, so the trailer keeps playing. Otherwise the trailer that is playing stops at the tap and none starts on the way;
            // the card on screen at the end takes it
            if distance > 40.0 {
                ui_store.spotlight_playing.set(None);
                ui_store.spotlight_hold.set(true);
                set_timeout(move || ui_store.spotlight_hold.set(false), std::time::Duration::from_millis(900));
                let opts = web_sys::ScrollToOptions::new();
                opts.set_top(top);
                opts.set_behavior(web_sys::ScrollBehavior::Smooth);
                win.scroll_to_with_scroll_to_options(&opts);
            }
        }
    });

    // Desktop rows from the engine; the fixed TMDB rows below are the fallback
    let desktop_feed = LocalResource::new(move || async move { crate::services::ai_engine::fetch_new_popular(None).await.unwrap_or_default() });

    view! {
        <Layout>
            <div class="relative min-h-screen bg-void md:bg-page">
                // Top spacing under fixed navbar: the first row title 20 px under the nav, as on Netflix's My Netflix (its New & Popular is still the old page)
                <div class="h-16 sm:h-20 md:h-[calc(var(--nav-h)+20px)]" />

                // ── MOBILE: ONE LONG PAGE OF TABS (< md) ───────────────────
                <div class="block md:hidden">
                    <div class="mobile-pills-sticky">
                        <NewPopularSubNavMobile active_tab=active_tab on_tab_change=jump_to />
                    </div>
                    <div class="px-3 pt-3 pb-24 space-y-8">
                        {SECTIONS.into_iter().map(|tab| if tab == NewPopularTab::Games { view! { <NewHotGames /> }.into_any() } else { view! { <NewHotSection tab=tab /> }.into_any() }).collect::<Vec<_>>()}
                    </div>
                </div>

                // ── DESKTOP DYNAMIC ROWS (>= md) ──────────────────────────
                <main class="hidden md:block relative z-10 pb-16 rows-stack">
                    <Suspense fallback=move || view! {
                        <div class="space-y-6">
                            <div class="h-40 bg-ink/[0.02] rounded-card animate-pulse mx-[var(--app-x,56px)]" />
                            <div class="h-40 bg-ink/[0.02] rounded-card animate-pulse mx-[var(--app-x,56px)]" />
                        </div>
                    }>
                        {move || desktop_feed.get().map(|rows| {
                            if rows.is_empty() {
                                view! { <FallbackRows /> }.into_any()
                            } else {
                                rows.into_iter().map(|r| {
                                    let row_title = r.display_title();
                                    let media_items = r.to_media_items();
                                    if r.is_top_ten() {
                                        let kind = if r.items.first().is_some_and(|i| i.is_tv()) { "tv" } else { "movie" };
                                        view! { <TopTenRow title=row_title kind=kind.to_string() items=Some(media_items) /> }.into_any()
                                    } else {
                                        view! {
                                            <crate::components::media::vibe_row::VibeRow
                                                title=row_title
                                                tagline=r.tagline.clone()
                                                mood_pills=r.mood_pills.clone()
                                                items=media_items
                                            />
                                        }.into_any()
                                    }
                                }).collect::<Vec<_>>().into_any()
                            }
                        })}
                    </Suspense>
                </main>
            </div>
        </Layout>
    }
}

/// The page as it was before the engine served it: fixed TMDB rows, shown only when the engine cannot be reached.
#[component]
fn FallbackRows() -> impl IntoView {
    view! {
        <TopTenRow title=crate::i18n::t("new-popular-top-10-films-in-the") kind="movie" />
        <TopTenRow title=crate::i18n::t("new-popular-top-10-series-in-the") kind="tv" />
        <Row title=crate::i18n::t("new-popular-newly-added-to-the-collection") endpoint="/movie/now_playing" kind="movie" />
        <Row title=crate::i18n::t("new-popular-worth-the-wait") sort_by="vote_average.desc" extra_params="&vote_count.gte=1000" kind="movie" />
        <Row title=crate::i18n::t("new-popular-rising-stars-under-the-radar") sort_by="popularity.desc" extra_params="&vote_count.lte=2500&vote_count.gte=150&vote_average.gte=7.0" kind="movie" />
        <Row title=crate::i18n::t("new-popular-the-best-of-2026-so") sort_by="vote_average.desc" extra_params="&primary_release_date.gte=2025-01-01&vote_count.gte=80" kind="movie" />
        <Row title=crate::i18n::t("new-popular-new-series-everyone-is-watching") endpoint="/trending/tv/week" kind="tv" />
        <Row title=crate::i18n::t("new-popular-the-films-everyone-is-discussing") endpoint="/trending/movie/week" kind="movie" />
        <Row title=crate::i18n::t("new-popular-coming-to-the-collection-soon") endpoint="/movie/upcoming" kind="movie" />
        <Row title=crate::i18n::t("new-popular-new-and-acclaimed") sort_by="vote_average.desc" extra_params="&vote_count.gte=400&primary_release_date.gte=2024-01-01" kind="movie" />
    }
}
