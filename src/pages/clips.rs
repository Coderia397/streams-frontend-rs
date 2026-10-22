use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use crate::services::tmdb::{fetch_trending, MediaItem};
use crate::store::{use_library_store, use_ui_store, LibraryEntry};
use crate::components::media::movie_card_badges::RatingBadge;

#[component]
pub fn ClipCard(
    item: MediaItem,
) -> impl IntoView {
    let navigate = use_navigate();
    let library = use_library_store();
    let ui_store = use_ui_store();

    let movie_id = item.id;
    let is_tv = item.is_tv();
    let title = item.display_title().to_string();
    let overview = item.overview.clone();
    let backdrop = item.backdrop_url("original").unwrap_or_default();
    let (is_muted, set_is_muted) = signal(true);

    let is_in_my_list = move || library.my_list.with(|m| m.contains_key(&movie_id));

    let item_clone = item.clone();
    let on_toggle_list = move |e: leptos::ev::MouseEvent| {
        e.stop_propagation();
        if is_in_my_list() {
            library.my_list.update(|m| { m.remove(&movie_id); });
        } else {
            let it = item_clone.clone();
            library.my_list.update(|m| {
                m.insert(movie_id, LibraryEntry {
                    media: it,
                    added_at: js_sys::Date::now() as u64,
                });
            });
        }
    };

    let on_play = move |e: leptos::ev::MouseEvent| {
        e.stop_propagation();
        let watch_store = crate::store::use_watch_store();
        let target = if let Some(rec) = watch_store.get_record(movie_id, is_tv) {
            if is_tv && rec.season.is_some() && rec.episode.is_some() {
                format!("/watch/tv/{}?season={}&episode={}", movie_id, rec.season.unwrap(), rec.episode.unwrap())
            } else if is_tv {
                format!("/watch/tv/{}", movie_id)
            } else {
                format!("/watch/movie/{}", movie_id)
            }
        } else if is_tv {
            format!("/watch/tv/{}", movie_id)
        } else {
            format!("/watch/movie/{}", movie_id)
        };
        navigate(&target, Default::default());
    };

    let on_card_tap = move |_| {
        set_is_muted.update(|m| *m = !*m);
    };

    let on_info_click = move |e: leptos::ev::MouseEvent| {
        e.stop_propagation();
        ui_store.hero_paused_by_modal.set(false);
        ui_store.modal_initial_time.set(0.0);
        ui_store.modal_current_time.set(0.0);
        ui_store.info_modal_movie_id.set(Some(movie_id));
        ui_store.info_modal_is_tv.set(is_tv);
        ui_store.info_modal_open.set(true);
    };

    view! {
        <section
            on:click=on_card_tap
            class="h-screen w-full snap-start relative flex flex-col justify-between overflow-hidden bg-void select-none"
        >
            // Full-screen backdrop
            {if !backdrop.is_empty() {
                view! {
                    <img
                        src=backdrop
                        alt=title.clone()
                        class="absolute inset-0 w-full h-full object-cover"
                        loading="lazy"
                    />
                }.into_any()
            } else {
                view! { <div class="absolute inset-0 bg-page" /> }.into_any()
            }}

            // Gradient scrims
            <div class="absolute inset-0 bg-gradient-to-b from-void/60 via-transparent to-void/80 pointer-events-none" />

            // Top Header: Back to browse
            <div class="relative z-20 pt-4 px-4 flex items-center justify-between">
                <a
                    href="/browse"
                    class="w-10 h-10 rounded-full bg-void/40 backdrop-blur-md flex items-center justify-center text-ink cursor-pointer"
                >
                    <i class="ph-bold ph-arrow-left text-xl"></i>
                </a>
                <span class="text-ink font-bold text-sm tracking-wider uppercase drop-shadow">
                    {crate::i18n::t("clips-clips")}
                </span>
                <div class="w-10" />
            </div>

            // Right-side actions column
            <div class="absolute right-3 bottom-24 z-30 flex flex-col items-center gap-5">
                // Mute toggle
                <button
                    on:click=move |e| {
                        e.stop_propagation();
                        set_is_muted.update(|m| *m = !*m);
                    }
                    class="flex flex-col items-center gap-1 text-ink cursor-pointer group"
                >
                    <div class="w-11 h-11 rounded-full bg-void/50 backdrop-blur-md flex items-center justify-center border border-ink/20 group-active:scale-90 transition-transform">
                        {move || if is_muted.get() {
                            view! { <i class="ph-fill ph-speaker-simple-slash text-xl text-ink"></i> }.into_any()
                        } else {
                            view! { <i class="ph-fill ph-speaker-simple-high text-xl text-ink"></i> }.into_any()
                        }}
                    </div>
                    <span class="text-caption font-semibold drop-shadow">
                        {move || if is_muted.get() { crate::i18n::t("clips-muted") } else { crate::i18n::t("clips-sound") }}
                    </span>
                </button>

                // My List toggle
                <button
                    on:click=on_toggle_list
                    class="flex flex-col items-center gap-1 text-ink cursor-pointer group"
                >
                    <div class="w-11 h-11 rounded-full bg-void/50 backdrop-blur-md flex items-center justify-center border border-ink/20 group-active:scale-90 transition-transform">
                        {move || if is_in_my_list() {
                            view! { <i class="ph-bold ph-check text-xl text-ink"></i> }.into_any()
                        } else {
                            view! { <i class="ph-bold ph-plus text-xl text-ink"></i> }.into_any()
                        }}
                    </div>
                    <span class="text-caption font-semibold drop-shadow">{crate::i18n::t("my-list")}</span>
                </button>

                // Play Button
                <button
                    on:click=on_play
                    class="flex flex-col items-center gap-1 text-ink cursor-pointer group"
                >
                    <div class="w-11 h-11 rounded-full bg-ink text-void flex items-center justify-center group-active:scale-90 transition-transform shadow-xl">
                        <i class="ph-fill ph-play text-xl ml-0.5 text-void"></i>
                    </div>
                    <span class="text-caption font-semibold drop-shadow">{move || {
                        let watch_store = crate::store::use_watch_store();
                        if let Some(rec) = watch_store.get_record(movie_id, is_tv) {
                            if rec.percentage > 0.0 {
                                return crate::i18n::t("media-resume");
                            }
                        }
                        crate::i18n::t("media-play")
                    }}</span>
                </button>

                // Info Button
                <button
                    on:click=on_info_click
                    class="flex flex-col items-center gap-1 text-ink cursor-pointer group"
                >
                    <div class="w-11 h-11 rounded-full bg-void/50 backdrop-blur-md flex items-center justify-center border border-ink/20 group-active:scale-90 transition-transform">
                        <i class="ph-bold ph-info text-xl text-ink"></i>
                    </div>
                    <span class="text-caption font-semibold drop-shadow">{crate::i18n::t("clips-info")}</span>
                </button>
            </div>

            // Bottom Content Overlay
            <div class="relative z-20 px-4 pb-8 max-w-[80%] flex flex-col gap-2">
                <div class="flex items-center gap-2 mb-1">
                    <RatingBadge id=movie_id is_tv=is_tv size="xs".to_string() />
                </div>
                <h2 class="text-ink font-black text-2xl md:text-3xl leading-tight drop-shadow-lg">
                    {title}
                </h2>
                <p class="text-ink/80 text-sm line-clamp-2 leading-relaxed font-normal drop-shadow">
                    {overview}
                </p>
            </div>
        </section>
    }
}

#[component]
pub fn ClipsPage() -> impl IntoView {
    let feed_resource = LocalResource::new(move || async move {
        // what is being watched today, from the engine (age limit applied there); TMDB only when the engine cannot be reached
        if let Some(rows) = crate::services::ai_engine::fetch_new_popular_n(Some("watching"), Some(20)).await {
            if let Some(row) = rows.first() {
                return row.to_media_items();
            }
        }
        fetch_trending("all").await.unwrap_or_default()
    });

    view! {
        <div class="fixed inset-0 bg-void overflow-y-scroll snap-y snap-mandatory scrollbar-hide z-50">
            <Suspense fallback=move || view! {
                <div class="fixed inset-0 bg-void flex items-center justify-center">
                    <div class="w-10 h-10 border-2 border-ink/20 border-t-ink rounded-full animate-spin"></div>
                </div>
            }>
                {move || feed_resource.get().map(|items| {
                    view! {
                        <>
                            {items.into_iter().map(|item| {
                                view! { <ClipCard item=item /> }
                            }).collect::<Vec<_>>()}
                        </>
                    }.into_any()
                })}
            </Suspense>
        </div>
    }
}
