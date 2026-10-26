use leptos::prelude::*;
use leptos_router::hooks::{use_navigate, use_params_map, use_query_map};
use crate::components::player::StreamsPlayer;
use crate::services::tmdb::fetch_details;
use crate::store::use_watch_store;

#[component]
pub fn PlayerPage() -> impl IntoView {
    let params = use_params_map();
    let query = use_query_map();
    let navigate = use_navigate();
    let watch_store = use_watch_store();

    let id_str = move || params.read().get("id").unwrap_or_default();
    let id_val = move || id_str().parse::<u32>().unwrap_or(0);

    // Initial query params
    let (active_season, set_active_season) = signal(1u32);
    let (active_episode, set_active_episode) = signal(1u32);

    let kind_param = move || params.read().get("kind");

    // Fetch movie or TV details, and whether the profile in use may watch the title: a watch address opened by hand, from a shared link or
    // from history on a Kids or age limited profile is checked with the same rule as every list (services::rating::filter_allowed)
    let media_resource = LocalResource::new(move || {
        let id = id_val();
        let k = kind_param();
        async move {
            let found = if id > 0 {
                let wanted = match k.as_deref() {
                    Some("tv") => fetch_details(id, true).await.ok().map(|d| (d, true)),
                    Some("movie") => fetch_details(id, false).await.ok().map(|d| (d, false)),
                    _ => None,
                };
                if wanted.is_some() {
                    wanted
                } else if let Ok(d) = fetch_details(id, false).await {
                    // no kind in the address: a film first, then a series
                    Some((d, false))
                } else if let Ok(d) = fetch_details(id, true).await {
                    Some((d, true))
                } else {
                    None
                }
            } else {
                None
            };
            let (d, is_tv) = found?;
            let item = crate::services::tmdb::MediaItem {
                id: d.id,
                media_type: Some(if is_tv { "tv" } else { "movie" }.to_string()),
                genre_ids: d.genres.as_ref().map(|g| g.iter().map(|x| x.id).collect()).unwrap_or_default(),
                ..Default::default()
            };
            let allowed = !crate::services::rating::filter_allowed(vec![item]).await.is_empty();
            Some((d, is_tv, allowed))
        }
    });

    // Tell the recommendation engine this title was started
    Effect::new(move |_| {
        let id = id_val();
        let kind = kind_param();
        if id > 0 {
            leptos::task::spawn_local(async move {
                crate::services::ai_engine::send_feedback(id, kind.as_deref(), "start").await;
            });
        }
    });

    // The season and episode: from the address, else from the saved progress, read once. Following the saved progress (it is written every
    // few seconds while playing) set them again and again, and each set built the whole player afresh: on an address without them
    // (/watch/tv/1421) the video went black and started again every 5 s (owner's recording, 2026-10-26, note 131). A value that is already
    // in use is not set again, for the same reason.
    let choose = move |s: u32, e: u32| {
        if active_season.get_untracked() != s { set_active_season.set(s); }
        if active_episode.get_untracked() != e { set_active_episode.set(e); }
    };
    Effect::new(move |_| {
        let q = query.read();
        let q_s = q.get("season").or_else(|| q.get("s")).and_then(|v| v.parse::<u32>().ok());
        let q_e = q.get("episode").or_else(|| q.get("e")).and_then(|v| v.parse::<u32>().ok());

        if let (Some(s), Some(e)) = (q_s, q_e) {
            choose(s, e);
        } else {
            let id = id_val();
            if let Some((Some(s), Some(e))) = untrack(|| watch_store.get_record(id, true)).map(|r| (r.season, r.episode)) {
                choose(s, e);
            }
        }
    });

    let nav_close = navigate.clone();
    let on_close = Callback::new(move |_| {
        nav_close("/browse", Default::default());
    });

    let nav_ep = navigate.clone();
    let on_episode_change = Callback::new(move |(s, e): (u32, u32)| {
        set_active_season.set(s);
        set_active_episode.set(e);
        let id = id_val();
        nav_ep(&format!("/watch/tv/{}?season={}&episode={}", id, s, e), Default::default());
    });

    view! {
        // the box of the form in use: the window on a computer; on a phone full screen in landscape (components/player/frame.rs)
        <crate::components::player::frame::PlayerFrame>
        <div class="w-full h-full bg-void overflow-hidden select-none">
            <Suspense fallback=move || view! {
                <div class="w-full h-full flex items-center justify-center bg-void" role="status" aria-label=crate::i18n::t("player-loading")>
                    // as on Netflix: black, one thin arc turning in the middle, 4vw wide, no text
                    <div class="aspect-square w-[4vw] animate-spin rounded-full border-[3px] border-transparent border-s-brand"></div>
                </div>
            }>
                {move || {
                    let id = id_val();
                    let s_num = active_season.get();
                    let e_num = active_episode.get();

                    media_resource.get().map(|res| match res {
                        Some((_, _, false)) => view! {
                            // above the profile's age rating (or not a children's title on a Kids profile): nothing is resolved or played
                            <div class="w-full h-full flex flex-col items-center justify-center bg-void text-center px-4">
                                <h2 class="text-xl font-bold text-ink mb-2">{crate::i18n::t("player-not-allowed")}</h2>
                                <p class="text-ink/60 text-sm mb-6">{crate::i18n::t("player-not-allowed-hint")}</p>
                                <button on:click=move |_| on_close.run(()) class="btn-brand px-6 py-2.5 text-sm">
                                    {crate::i18n::t("controls-overlay-back-to-browse")}
                                </button>
                            </div>
                        }.into_any(),
                        Some((item, is_tv, true)) => {
                            let title = item.display_title().to_string();
                            let year = item.release_date.as_ref().or(item.first_air_date.as_ref()).map(|d| {
                                d.chars().take(4).collect::<String>()
                            });
                            let poster = item.poster_path.clone();
                            let backdrop = item.backdrop_path.clone();
                            let orig_lang = item.original_language.clone();

                            view! {
                                <StreamsPlayer
                                    media_id=id
                                    title=title
                                    is_tv=is_tv
                                    season=if is_tv { Some(s_num) } else { None }
                                    episode=if is_tv { Some(e_num) } else { None }
                                    episode_title=None
                                    year=year
                                    orig_lang=orig_lang
                                    poster_path=poster
                                    backdrop_path=backdrop
                                    on_close=on_close
                                    on_episode_change=Some(on_episode_change)
                                />
                            }.into_any()
                        }
                        None => view! {
                            <div class="w-full h-full flex flex-col items-center justify-center bg-void text-center px-4">
                                <h2 class="text-xl font-bold text-ink mb-2">{crate::i18n::t("player-not-found")}</h2>
                                <p class="text-ink/60 text-sm mb-6">{crate::i18n::t("player-not-found-hint")}</p>
                                <button
                                    on:click=move |_| on_close.run(())
                                    class="btn-brand px-6 py-2.5 text-sm"
                                >
                                    {crate::i18n::t("controls-overlay-back-to-browse")}
                                </button>
                            </div>
                        }.into_any()
                    })
                }}
            </Suspense>
        </div>
        </crate::components::player::frame::PlayerFrame>
    }
}

