use leptos::prelude::*;
use crate::models::movie::Movie;
use crate::services::tmdb::{fetch_recommendations, fetch_movie_logo};
use crate::components::media::movie_card_badges::RatingBadge;
use crate::components::media::action_buttons::MyListButton;
use crate::store::{use_library_store, LibraryEntry};

const BATCH_REC: usize = 9;

#[component]
pub fn RecCard(
    rec: Movie,
    on_play: Option<Callback<Movie>>,
    on_open_modal: Callback<Movie>,
) -> impl IntoView {
    let library_store = use_library_store();
    let rec_id = rec.id_u32();
    let is_tv = rec.is_tv();
    let title = rec.display_title();
    let match_score = rec.match_score();
    let year = rec.release_date.as_ref().or(rec.first_air_date.as_ref())
        .map(|d| d.chars().take(4).collect::<String>()).unwrap_or_default();
    let overview = if rec.overview.is_empty() {
        crate::i18n::t("media-no-description")
    } else {
        rec.overview.clone()
    };

    let is_added = Signal::derive(move || {
        library_store.my_list.get().contains_key(&rec_id)
    });

    let toggle_my_list_cb = {
        let rec_clone = rec.clone();
        Callback::new(move |_| {
            let mut list = library_store.my_list.get();
            if list.contains_key(&rec_id) {
                list.remove(&rec_id);
            } else {
                list.insert(rec_id, LibraryEntry {
                    media: crate::services::tmdb::MediaItem {
                        id: rec_id,
                        title: rec_clone.title.clone(),
                        name: rec_clone.name.clone(),
                        overview: rec_clone.overview.clone(),
                        poster_path: rec_clone.poster_path.clone(),
                        backdrop_path: rec_clone.backdrop_path.clone(),
                        vote_average: rec_clone.vote_average,
                        release_date: rec_clone.release_date.clone(),
                        first_air_date: rec_clone.first_air_date.clone(),
                        media_type: rec_clone.media_type.clone(),
                        ..Default::default()
                    },
                    added_at: js_sys::Date::now() as u64,
                });
            }
            library_store.my_list.set(list);
        })
    };

    let logo_resource = LocalResource::new(move || {
        let mid = rec_id;
        let is_tv_val = is_tv;
        async move {
            if mid > 0 {
                fetch_movie_logo(mid, is_tv_val).await.ok().flatten()
            } else {
                None
            }
        }
    });

    let backdrop = rec.backdrop_path.as_ref().or(rec.poster_path.as_ref())
        .map(|p| {
            if p.starts_with("http") {
                p.clone()
            } else {
                format!("{}/w780{}", crate::config::TMDB_IMAGE_URL, p)
            }
        });

    let on_image_click = {
        let rec_c = rec.clone();
        move |_| {
            if let Some(cb) = on_play {
                cb.run(rec_c.clone());
            } else if let Some(w) = web_sys::window() {
                let kind = if is_tv { "tv" } else { "movie" };
                let _ = w.location().set_href(&format!("/watch/{}/{}", kind, rec_id));
            }
        }
    };

    let on_body_click = {
        let rec_c = rec.clone();
        move |_| {
            on_open_modal.run(rec_c.clone());
        }
    };

    // the shared content card (as the title page's): the picture inset 8 px with its own corners, a round Play over it, then the line of
    // facts with the My List button, and the synopsis
    view! {
        <div class="surface-card surface-card-hover group flex flex-col overflow-hidden p-2">
            // picture: click to play
            <div class="relative aspect-video cursor-pointer overflow-hidden rounded-xl bg-raised" on:click=on_image_click>
                {if let Some(src) = backdrop {
                    view! {
                        <img
                            src=src
                            alt=title.clone()
                            class="h-full w-full object-cover transition-transform duration-(--motion-base) group-hover:scale-[1.03]"
                            loading="lazy"
                        />
                    }.into_any()
                } else {
                    view! { <div class="h-full w-full bg-placeholder" /> }.into_any()
                }}
                <div class="pointer-events-none absolute inset-0 bg-linear-to-t from-void/60 via-transparent to-transparent"></div>

                // Play: the round preset, shown under the pointer (always on a touch screen, which has no pointer to rest)
                <div class="pointer-events-none absolute inset-0 flex items-center justify-center opacity-0 transition-opacity duration-(--motion-base) group-hover:opacity-100 [@media(hover:none)]:opacity-100">
                    <span class="btn-circle btn-circle-lg"><i class="ph-fill ph-play"></i></span>
                </div>

                // the title's logo, or its name
                <div class="absolute bottom-2.5 start-3 end-3 max-w-[78%]">
                    <Suspense fallback=move || view! { <div class="h-6 w-20 animate-pulse rounded-tick bg-ink/10" /> }>
                        {
                            let tc = title.clone();
                            move || {
                                let t = tc.clone();
                                logo_resource.get().map(|res| match res {
                                    Some(url) if !url.is_empty() => view! {
                                        <img src=url alt=t.clone() class="max-h-11 max-w-full object-contain object-left-bottom drop-shadow-lg rtl:object-right-bottom" />
                                    }.into_any(),
                                    _ => view! {
                                        <p class="line-clamp-2 text-[16px] font-bold leading-tight text-ink drop-shadow-md">{t.clone()}</p>
                                    }.into_any(),
                                })
                            }
                        }
                    </Suspense>
                </div>
            </div>

            // facts and synopsis: click opens the title's details
            <div class="flex flex-1 cursor-pointer flex-col px-2 pb-2 pt-3" on:click=on_body_click>
                <div class="flex items-center justify-between gap-2">
                    <div class="flex min-w-0 items-center gap-x-1.5 overflow-hidden whitespace-nowrap text-[14px] font-medium leading-[21px]">
                        <span class="text-success">
                            {crate::i18n::t_args("media-percent-match", &[("pct", fluent_bundle::FluentValue::from(match_score as i64))])}
                        </span>
                        <RatingBadge id=rec.id_u32() is_tv=rec.is_tv() gb=rec.rating_gb.clone() us=rec.rating_us.clone() size="xs".to_string() />
                        {(!year.is_empty()).then(|| view! { <span class="text-ink/70">{year}</span> })}
                    </div>
                    <MyListButton is_in_list=is_added on_toggle=toggle_my_list_cb size="sm".to_string() />
                </div>
                <p class="mt-2 line-clamp-4 text-[14px] leading-[21px] text-ink/70">{overview}</p>
            </div>
        </div>
    }
}

#[component]
pub fn InfoModalRecommendations(
    movie_id: u32,
    is_tv: bool,
    on_recommendation_click: Callback<Movie>,
    #[prop(optional)] on_play: Option<Callback<Movie>>,
) -> impl IntoView {
    let visible_count = RwSignal::new(BATCH_REC);

    let rec_resource = LocalResource::new(move || {
        let mid = movie_id;
        let tv = is_tv;
        async move {
            if mid > 0 {
                // First try local vector similarity recommendations
                if let Some(ai_items) = crate::services::ai_engine::fetch_ai_recommendations(mid, tv).await {
                    if !ai_items.is_empty() {
                        let items: Vec<crate::services::tmdb::MediaItem> = ai_items.into_iter().map(|item| item.to_media_item()).collect();
                        return Some(items);
                    }
                }
                fetch_recommendations(mid, tv).await.ok()
            } else {
                None
            }
        }
    });

    view! {
        <Suspense fallback=move || view! {
            <div class="mt-12">
                <h3 class="mb-4 text-[24px] font-medium leading-[1.25] text-ink">{crate::i18n::t("media-more-like-this")}</h3>
                <div class="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">
                    {(0..6).map(|_| view! {
                        <div class="surface-card aspect-[4/5] animate-pulse" />
                    }).collect_view()}
                </div>
            </div>
        }>
            {move || rec_resource.get().map(|res| match res {
                Some(items) if !items.is_empty() => {
                    let total = items.len();
                    let count = visible_count.get();
                    let visible_items: Vec<_> = items.into_iter().take(count).collect();
                    let has_more = count < total;

                    view! {
                        <div class="mt-12">
                            <h3 class="mb-4 text-[24px] font-medium leading-[1.25] text-ink">{crate::i18n::t("media-more-like-this")}</h3>
                            <div class="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">
                                {visible_items.into_iter().map(|item| {
                                    let movie = Movie::from(item);
                                    let on_rec_click = on_recommendation_click;
                                    let on_play_cb = on_play;

                                    view! {
                                        <RecCard
                                            rec=movie
                                            on_play=on_play_cb
                                            on_open_modal=on_rec_click
                                        />
                                    }
                                }).collect_view()}
                            </div>

                            {if has_more || count > BATCH_REC {
                                view! {
                                    <div class="flex justify-center mt-5">
                                        <button
                                            type="button"
                                            on:click=move |e| {
                                                e.stop_propagation();
                                                if has_more {
                                                    visible_count.update(|c| *c += BATCH_REC);
                                                } else {
                                                    visible_count.set(BATCH_REC);
                                                }
                                            }
                                            class="btn-circle btn-circle-md group/btn"
                                        >
                                            <i class=if has_more {
                                                "ph-bold ph-caret-down text-ink/60 group-hover/btn:text-ink transition-colors"
                                            } else {
                                                "ph-bold ph-caret-up text-ink/60 group-hover/btn:text-ink transition-colors"
                                            }></i>
                                        </button>
                                    </div>
                                }.into_any()
                            } else {
                                view! { <span /> }.into_any()
                            }}
                        </div>
                    }.into_any()
                },
                _ => view! { <span /> }.into_any()
            })}
        </Suspense>
    }
}
