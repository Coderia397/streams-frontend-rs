use leptos::prelude::*;
use crate::services::tmdb::{fetch_details, fetch_season_details, Episode};
use crate::store::use_watch_store;

/// The episodes panel, laid out from the owner's Netflix screenshots (note 70): a 760 px dark panel above the controls on the right; a header with a back arrow and "Season N" (the arrow
/// opens the list of seasons); one row per episode with its number, name and a thin progress line; the current episode (and the row that has the keyboard focus) opens into a
/// thumbnail with four lines of the summary, the current one marked "Now Playing". Progress is known for the episode in the watch record only, the other lines stay empty.
#[component]
pub fn EpisodeDrawer(
    series_id: u32,
    current_season: u32,
    current_episode: u32,
    is_open: ReadSignal<bool>,
    on_close: Callback<()>,
    on_select_episode: Callback<(u32, u32)>,
) -> impl IntoView {
    let watch_store = use_watch_store();
    let (selected_season, set_selected_season) = signal(current_season);
    let (show_seasons, set_show_seasons) = signal(false);

    let seasons_resource = LocalResource::new(move || async move {
        if series_id > 0 { fetch_details(series_id, true).await.ok().and_then(|d| d.number_of_seasons) } else { None }
    });
    let episodes_resource = LocalResource::new(move || {
        let season_num = selected_season.get();
        async move { if series_id > 0 { fetch_season_details(series_id, season_num).await.ok().map(|d| d.episodes) } else { None } }
    });
    let many_seasons = move || seasons_resource.get().flatten().unwrap_or(1) > 1;

    let row_base = "group relative flex w-full flex-col px-[35px] text-start transition-colors duration-(--motion-quick) outline-none";
    view! {
        <Show when=move || is_open.get()>
            // a click anywhere else closes the panel (no dimming)
            <div on:click=move |_| on_close.run(()) class="fixed inset-0 z-40" />

            <div class="fixed bottom-[94px] right-0 z-50 flex h-[calc(100vh-253px)] min-h-[320px] w-[min(760px,100vw)] flex-col overflow-hidden bg-field shadow-2xl">
                // header: back arrow and the season
                <button
                    on:click=move |_| if many_seasons() { set_show_seasons.update(|s| *s = !*s) }
                    class="flex h-[74px] w-full shrink-0 items-center gap-[18px] px-[24px] text-start text-ink transition-colors duration-(--motion-quick) hover:bg-line"
                >
                    <Show when=many_seasons><i class="ph ph-arrow-left text-[26px]"></i></Show>
                    <span class="text-[30px] font-medium leading-[36px]">
                        {move || crate::i18n::t_args("media-season-n", &[("n", fluent_bundle::FluentValue::from(selected_season.get() as i64))])}
                    </span>
                </button>

                <div class="flex-1 overflow-y-auto">
                    // list of seasons
                    <Show when=move || show_seasons.get()>
                        {move || (1..=seasons_resource.get().flatten().unwrap_or(1)).map(|n| view! {
                            <button
                                on:click=move |_| { set_selected_season.set(n); set_show_seasons.set(false); }
                                class=format!("{row_base} h-[85px] justify-center text-[23px] font-medium hover:bg-[#404040] {}", if n == current_season { "text-ink" } else { "text-[#d2d2d2]" })
                            >
                                {crate::i18n::t_args("media-season-n", &[("n", fluent_bundle::FluentValue::from(n as i64))])}
                            </button>
                        }).collect::<Vec<_>>()}
                    </Show>

                    // list of episodes
                    <Show when=move || !show_seasons.get()>
                        <Suspense fallback=move || view! {
                            <div class="flex items-center justify-center py-20"><div class="aspect-square w-[40px] animate-spin rounded-full border-[3px] border-transparent border-s-brand"></div></div>
                        }>
                            {move || episodes_resource.get().map(|res| match res {
                                Some(episodes) if !episodes.is_empty() => episodes.into_iter().map(|ep: Episode| {
                                    let ep_num = ep.episode_number;
                                    let s_num = selected_season.get();
                                    let is_current = s_num == current_season && ep_num == current_episode;
                                    let still_url = ep.still_url("w300");
                                    let name = ep.name.clone();
                                    let overview = ep.overview.clone();
                                    let percent = watch_store.episode_percent(series_id, s_num, ep_num).clamp(0.0, 100.0);
                                    let open_class = if is_current { "bg-raised" } else { "hover:bg-[#404040] focus-visible:bg-raised focus-visible:outline-2 focus-visible:outline-ink" };
                                    view! {
                                        <button
                                            on:click=move |_| { on_select_episode.run((s_num, ep_num)); on_close.run(()); }
                                            class=format!("{row_base} {open_class}")
                                        >
                                            // number, name and the progress line
                                            <span class=format!("flex w-full items-center gap-[24px] text-[23px] font-medium text-ink {}", if is_current { "pt-[22px]" } else { "h-[85px] group-focus-visible:pt-[22px] group-focus-visible:h-auto" })>
                                                <span class="w-[26px] shrink-0 text-center">{ep_num}</span>
                                                <span class="min-w-0 flex-1 truncate">{name.clone()}</span>
                                                <span class="relative h-[2px] w-[131px] shrink-0 bg-mute"><span class="absolute inset-y-0 left-0 bg-brand" style=format!("width:{percent}%")></span></span>
                                            </span>
                                            // the open part: thumbnail and four lines of the summary
                                            <span class=format!("w-full gap-[26px] pb-[22px] pt-[12px] {}", if is_current { "flex" } else { "hidden group-focus-visible:flex" })>
                                                <span class="relative h-[140px] w-[248px] shrink-0 overflow-hidden bg-[#1a1a1a]">
                                                    {still_url.map(|u| view! { <img src=u alt=name.clone() class="h-full w-full object-cover" loading="lazy" /> })}
                                                    <span class="absolute inset-0 flex items-center justify-center bg-void/30 text-ink">
                                                        {if is_current {
                                                            view! { <span class="flex items-center gap-2 text-[22px] font-medium"><i class="ph-fill ph-waveform text-[26px]"></i>{crate::i18n::t("player-now-playing")}</span> }.into_any()
                                                        } else {
                                                            view! { <span class="flex h-[64px] w-[64px] items-center justify-center rounded-full bg-void/60"><i class="ph-fill ph-play text-[34px]"></i></span> }.into_any()
                                                        }}
                                                    </span>
                                                </span>
                                                <span class="line-clamp-4 block max-h-[100px] min-w-0 flex-1 overflow-hidden text-[20px] leading-[25px] text-ink">{overview}</span>
                                            </span>
                                        </button>
                                    }
                                }).collect::<Vec<_>>().into_any(),
                                _ => view! { <div class="py-16 text-center text-[18px] text-[#b3b3b3]">{crate::i18n::t("episode-drawer-no-episodes-available-for-this")}</div> }.into_any(),
                            })}
                        </Suspense>
                    </Show>
                </div>
            </div>
        </Show>
    }
}
