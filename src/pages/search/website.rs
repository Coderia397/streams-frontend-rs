//! Search on tablets and computers (note 114): the top bar's field is the search; the recommendations and the results are grids of
//! tiles, the results under Netflix's "More to explore" line.
use super::{Found, Recommended};
use crate::components::media::movie_card::MovieCard;
use crate::services::tmdb::MediaItem;
use leptos::prelude::*;

/// The grid's place holders while the titles load: the grid's own columns.
fn grid_skeleton() -> impl IntoView {
    view! { <crate::components::skeleton::GridSkeleton columns="grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 gap-x-2 gap-y-5" /> }
}

#[component]
pub fn WebsiteSearch(#[prop(into)] q: Signal<String>, results: Found, recommended: Recommended) -> impl IntoView {
    view! {
        <div class="pt-28 px-[var(--app-x,56px)] pb-12 min-h-screen bg-page">
            // Nothing typed: the recommendations
            <Show when=move || q.get().trim().is_empty()>
                <h1 class="text-title-md font-bold text-ink mb-3 mt-1 ps-1">{crate::i18n::t("search-recommended")}</h1>
                <Suspense fallback=grid_skeleton>
                    {move || recommended.get().map(|items| match items {
                        Some(items) if !items.is_empty() => view! {
                            <div class="grid grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 gap-x-2 gap-y-5">
                                {items.into_iter().take(15).map(tile).collect::<Vec<_>>()}
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

            // Something typed: the grid, under Netflix's "More to explore" line (16 px; the titles found that carry the words searched, each
            // a search of its own, 8 px either side of a thin bar; the grid 24 px below, measured 2026-10-23)
            <Show when=move || !q.get().trim().is_empty()>
                <Suspense fallback=grid_skeleton>
                    {move || results.get().map(|media| {
                        if media.is_empty() {
                            return view! {
                                <div class="flex flex-col items-center justify-center mt-24 text-center px-6">
                                    <div class="text-title-md font-bold text-ink mb-2">{crate::i18n::t("search-no-matches")}</div>
                                    <div class="text-ink/60 text-base">{crate::i18n::t("search-try-another")}</div>
                                </div>
                            }.into_any();
                        }
                        let query = q.get_untracked();
                        let related = more_to_explore(&media, &query);
                        view! {
                            <h1 class="sr-only">{crate::i18n::t_args("search-results-for", &[("query", fluent_bundle::FluentValue::from(query))])}</h1>
                            {(!related.is_empty()).then(|| view! {
                                <div class="mb-6 flex items-start gap-2 text-body-md leading-6">
                                    <span class="shrink-0 text-ink/50">{crate::i18n::t("search-more-to-explore")}</span>
                                    <ul class="flex flex-wrap gap-y-3">
                                        {related.into_iter().map(|name| {
                                            let href = format!("/search?q={}", String::from(js_sys::encode_uri_component(&name)));
                                            view! { <li class="me-2 border-e border-ink/50 pe-2 last:me-0 last:border-e-0 last:pe-0"><a href=href class="text-ink hover:underline">{name}</a></li> }
                                        }).collect::<Vec<_>>()}
                                    </ul>
                                </div>
                            })}
                            <div class="grid grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 gap-x-2 gap-y-5">
                                {media.into_iter().map(tile).collect::<Vec<_>>()}
                            </div>
                        }.into_any()
                    })}
                </Suspense>
            </Show>
        </div>
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

/// A tile of the grid.
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
