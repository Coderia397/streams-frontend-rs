use leptos::prelude::*;
use crate::services::tmdb::fetch_season_details;

const BATCH_SIZE: usize = 6;

#[component]
pub fn InfoModalEpisodes(
    series_id: u32,
    total_seasons: u32,
    selected_season: RwSignal<u32>,
    #[prop(optional)] on_play: Option<Callback<(u32, u32, u32)>>,
) -> impl IntoView {
    let dropdown_open = RwSignal::new(false);
    let visible_count = RwSignal::new(BATCH_SIZE);

    // Reset visible count when season changes
    Effect::new(move |_| {
        let _ = selected_season.get();
        visible_count.set(BATCH_SIZE);
    });

    let season_resource = LocalResource::new(move || {
        let sid = series_id;
        let season_num = selected_season.get();
        async move {
            if sid > 0 && season_num > 0 {
                fetch_season_details(sid, season_num).await.ok()
            } else {
                None
            }
        }
    });

    let toggle_dropdown = move |e: leptos::ev::MouseEvent| {
        e.stop_propagation();
        dropdown_open.update(|open| *open = !*open);
    };

    view! {
        <div class="mt-8">
            // Header: "Episodes" title and Season dropdown
            <div class="flex items-center justify-between mb-4">
                <h3 class="text-2xl font-medium text-ink">{crate::i18n::t("media-episodes")}</h3>

                {if total_seasons > 0 {
                    view! {
                        <div class="relative">
                            <button
                                type="button"
                                on:click=toggle_dropdown
                                class="flex items-center bg-transparent border border-ink/30 hover:border-ink/70 rounded-button px-4 py-1.5 text-sm font-bold transition min-w-[130px] justify-between text-ink cursor-pointer"
                            >
                                <span>{move || crate::i18n::t_args("media-season-n", &[("n", fluent_bundle::FluentValue::from(selected_season.get() as i64))])}</span>
                                <i class=move || if dropdown_open.get() { "ph-bold ph-caret-up ml-2 text-xs" } else { "ph-bold ph-caret-down ml-2 text-xs" }></i>
                            </button>

                            {move || if dropdown_open.get() {
                                view! {
                                    <div class="absolute right-0 top-full mt-1 w-32 bg-field border border-gray-700 rounded-menu shadow-xl z-50 max-h-60 overflow-y-auto scrollbar-hide">
                                        {(1..=total_seasons).map(|s| {
                                            let is_selected = move || selected_season.get() == s;
                                            view! {
                                                <div
                                                    on:click=move |e| {
                                                        e.stop_propagation();
                                                        selected_season.set(s);
                                                        dropdown_open.set(false);
                                                    }
                                                    class="px-4 py-2 text-sm cursor-pointer hover:bg-lift-strong transition-colors"
                                                    class=("bg-line", is_selected)
                                                    class=("font-bold", is_selected)
                                                    class=("text-ink", is_selected)
                                                    class=("text-gray-300", move || !is_selected())
                                                >
                                                    {crate::i18n::t_args("media-season-n", &[("n", fluent_bundle::FluentValue::from(s as i64))])}
                                                </div>
                                            }
                                        }).collect_view()}
                                    </div>
                                }.into_any()
                            } else {
                                view! { <span /> }.into_any()
                            }}
                        </div>
                    }.into_any()
                } else {
                    view! { <span /> }.into_any()
                }}
            </div>

            // Episode list
            <div class="space-y-1">
                <Suspense fallback=move || view! {
                    <div class="space-y-2">
                        {(0..4).map(|_| view! {
                            <div class="flex items-start p-4 rounded-control border-b border-ink/5 gap-4">
                                <div class="w-8 h-5 bg-ink/[0.06] rounded-control animate-pulse shrink-0" />
                                <div class="w-28 md:w-36 h-16 md:h-20 bg-ink/[0.06] rounded-control animate-pulse shrink-0" />
                                <div class="flex-1 space-y-2 py-1">
                                    <div class="h-3 bg-ink/[0.08] rounded-full w-2/3 animate-pulse" />
                                    <div class="h-2.5 bg-ink/[0.05] rounded-full w-full animate-pulse" />
                                    <div class="h-2.5 bg-ink/[0.05] rounded-full w-4/5 animate-pulse" />
                                </div>
                            </div>
                        }).collect_view()}
                    </div>
                }>
                    {move || season_resource.get().map(|res| match res {
                        Some(details) if !details.episodes.is_empty() => {
                            let episodes = details.episodes;
                            let total_eps = episodes.len();
                            let current_count = visible_count.get();
                            let visible_episodes: Vec<_> = episodes.into_iter().take(current_count).collect();
                            let has_more = current_count < total_eps;
                            let season_val = selected_season.get();

                            view! {
                                <>
                                    {visible_episodes.into_iter().map(|ep| {
                                        let ep_num = ep.episode_number;
                                        let ep_name = ep.name.clone();
                                        let ep_overview = if ep.overview.is_empty() {
                                            crate::i18n::t("media-no-description")
                                        } else {
                                            ep.overview.clone()
                                        };
                                        let runtime_str = ep.runtime.map(|r| format!("{}m", r)).unwrap_or_default();
                                        let still_url = ep.still_url("w300");
                                        let watched = crate::store::use_watch_store().episode_percent(series_id, season_val, ep_num).clamp(0.0, 100.0);

                                        let on_click_ep = {
                                            let on_play_cb = on_play;
                                            move |_| {
                                                if let Some(cb) = on_play_cb {
                                                    cb.run((series_id, season_val, ep_num));
                                                } else if let Some(w) = web_sys::window() {
                                                    let _ = w.location().set_href(&format!("/watch/tv/{}?season={}&episode={}", series_id, season_val, ep_num));
                                                }
                                            }
                                        };

                                        view! {
                                            <div
                                                on:click=on_click_ep
                                                class="flex items-center group cursor-pointer p-4 rounded-control hover:bg-line transition"
                                            >
                                                // Episode number
                                                <div class="text-ink-light text-2xl font-normal w-[51px] text-center shrink-0">
                                                    {ep_num}
                                                </div>

                                                // Thumbnail + play icon overlay
                                                <div class="relative w-[130px] h-[73px] bg-gray-800 shrink-0 rounded-control overflow-hidden mr-4">
                                                    {if let Some(url) = still_url {
                                                        view! {
                                                            <img
                                                                src=url
                                                                class="w-full h-full object-cover group-hover:brightness-50 transition duration-(--motion-base)"
                                                                alt=ep_name.clone()
                                                                loading="lazy"
                                                            />
                                                        }.into_any()
                                                    } else {
                                                        view! {
                                                            <div class="w-full h-full flex items-center justify-center text-gray-600 text-xs">
                                                                {crate::i18n::t("info-modal-episodes-no-image")}
                                                            </div>
                                                        }.into_any()
                                                    }}
                                                    {(watched > 0.0).then(|| view! {
                                                        <div class="absolute inset-x-0 bottom-0 h-[3px] bg-mute"><div class="h-full bg-brand" style=format!("width:{watched}%")></div></div>
                                                    })}
                                                    <div class="absolute inset-0 flex items-center justify-center">
                                                        <div class="w-9 h-9 rounded-full bg-void/30 border border-ink/60 flex items-center justify-center group-hover:bg-ink/20 group-hover:scale-110 transition-all duration-(--motion-base) shadow">
                                                            <div class="w-0 h-0 ml-0.5 border-t-[6px] border-t-transparent border-b-[6px] border-b-transparent border-l-[10px] border-l-ink" />
                                                        </div>
                                                    </div>
                                                </div>

                                                // Episode info
                                                <div class="flex-1 min-w-0 py-0.5">
                                                    <div class="flex items-center justify-between mb-1">
                                                        <h4 class="text-ink font-medium text-body-md truncate pr-4">{ep_name}</h4>
                                                        <span class="text-ink text-body-md whitespace-nowrap shrink-0">{runtime_str}</span>
                                                    </div>
                                                    <p class="text-ink-light text-body-sm leading-5 line-clamp-3">
                                                        {ep_overview}
                                                    </p>
                                                </div>
                                            </div>
                                        }
                                    }).collect_view()}

                                    // Show more / show less toggle
                                    {if has_more || current_count > BATCH_SIZE {
                                        view! {
                                            <div class="flex justify-center mt-4">
                                                <button
                                                    type="button"
                                                    on:click=move |e| {
                                                        e.stop_propagation();
                                                        if has_more {
                                                            visible_count.update(|c| *c += BATCH_SIZE);
                                                        } else {
                                                            visible_count.set(BATCH_SIZE);
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
                                </>
                            }.into_any()
                        },
                        _ => view! {
                            <div class="text-gray-500 text-center py-6">{crate::i18n::t("media-no-episodes")}</div>
                        }.into_any()
                    })}
                </Suspense>
            </div>
        </div>
    }
}
