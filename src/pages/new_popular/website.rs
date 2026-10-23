//! New & Popular on tablets and computers (note 114): the engine's rows (its Top 10s as Top 10 rows), the fixed TMDB rows when the
//! engine cannot be reached.
use crate::components::media::row::Row;
use crate::components::media::top_ten_row::TopTenRow;
use leptos::prelude::*;

#[component]
pub fn WebsiteNewPopular() -> impl IntoView {
    let desktop_feed = LocalResource::new(move || async move { crate::services::ai_engine::fetch_new_popular(None).await.unwrap_or_default() });
    view! {
        <div class="relative min-h-screen bg-page">
            // the first row title 20 px under the nav, as on Netflix's My Netflix (its New & Popular is still the old page)
            <div class="h-[calc(var(--nav-h)+20px)]" />
                <main class="relative z-10 pb-16 rows-stack">
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
