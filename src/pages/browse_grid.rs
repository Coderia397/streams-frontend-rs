use leptos::prelude::*;
use leptos_router::hooks::{use_navigate, use_params_map, use_query_map};
use crate::components::layout::Layout;
use crate::components::media::movie_card::MovieCard;
use crate::models::mapping::map_genre_key_to_tmdb;
use crate::components::media::icons::app_icons::{AppIcon, Glyph};

#[component]
pub fn BrowseGridPage() -> impl IntoView {
    let navigate = use_navigate();
    let params = use_params_map();
    let query = use_query_map();

    let row_key = move || params.read().get("id").or_else(|| params.read().get("rowKey")).unwrap_or_default();
    let title_param = move || query.read().get("title");

    let display_title = move || {
        if let Some(t) = title_param() {
            return t;
        }
        let key = row_key();
        if let Some(ctx) = map_genre_key_to_tmdb(&key) {
            return ctx.title.to_string();
        }
        key.split('-')
            .map(|w| {
                let mut c = w.chars();
                match c.next() {
                    None => String::new(),
                    Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    };

    let items_resource = LocalResource::new(move || {
        let key = row_key();
        let q_kind = query.read().get("kind").or_else(|| query.read().get("type")).unwrap_or_else(|| "movie".to_string());
        async move {
            if key.is_empty() {
                vec![]
            } else {
                crate::services::tmdb::fetch_row_content(&q_kind, Some(&key), None, None, None, 1).await.unwrap_or_default()
            }
        }
    });

    let on_back = move |_| {
        if let Some(win) = web_sys::window() {
            let _ = win.history().map(|h| h.back());
        } else {
            navigate("/browse", Default::default());
        }
    };

    view! {
        <Layout>
            <div class="bg-void md:bg-page min-h-screen pb-16 pt-[calc(4.5rem+env(safe-area-inset-top))] md:pt-24">
                <div class="px-6 md:px-14">
                    // Header with back button
                    <div class="flex items-center gap-3.5 mb-8 md:mb-12">
                        <button
                            on:click=on_back
                            class="btn-circle btn-circle-clear btn-circle-md"
                            aria-label=crate::i18n::t("browse-grid-go-back")
                        >
                            <AppIcon glyph=Glyph::Back class="text-2xl md:text-3xl text-ink" />
                        </button>
                        <h1 class="text-ink font-bold text-2xl md:text-4xl tracking-tight">
                            {move || display_title()}
                        </h1>
                    </div>

                    // Content grid
                    <Suspense fallback=move || view! { <crate::components::skeleton::GridSkeleton columns="grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 gap-x-1.5 md:gap-x-2 gap-y-10 md:gap-y-12" count=18 /> }>
                        {move || items_resource.get().map(|items| {
                            if items.is_empty() {
                                view! {
                                    <div class="flex flex-col items-center justify-center mt-24 text-center">
                                        <i class="ph ph-film-slate text-5xl text-ink/30 mb-3"></i>
                                        <p class="text-ink/50 text-lg">{crate::i18n::t("browse-grid-empty")}</p>
                                    </div>
                                }.into_any()
                            } else {
                                view! {
                                    <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 gap-x-1.5 md:gap-x-2 gap-y-10 md:gap-y-12">
                                        {items.into_iter().map(|item| {
                                            let movie_id = item.id;
                                            let is_tv = item.is_tv();
                                            let title = item.display_title().to_string();
                                            let backdrop = item.backdrop_url("w780").unwrap_or_default();
                                            let poster = item.poster_url("w342").unwrap_or_default();
                                            let vote_avg = item.vote_average;
                                            view! {
                                                <MovieCard
                                                    movie_id=movie_id
                                                    is_tv=is_tv
                                                    title=title
                                                    backdrop_path=backdrop
                                                    poster_path=poster
                                                    vote_average=vote_avg
                                                    overview=item.overview.clone()
                                                    genre_ids=item.genre_ids.clone()
                                                    vibe_pills=item.vibe_pills.clone()
                                                    is_grid=true
                                                />
                                            }
                                        }).collect::<Vec<_>>()}
                                    </div>
                                }.into_any()
                            }
                        })}
                    </Suspense>
                </div>
            </div>
        </Layout>
    }
}
