//! Episodes on the phone's player, as Netflix's Android app (view tree and frames on a Pixel 8, 2026-10-24, note 116): the whole
//! screen, a back arrow at the top left and the season at the top right (it opens the season menu); the season's episodes in a row
//! that scrolls sideways, cards 234 px wide and 20 px apart from 70 px in: the still (16:9, a play mark on the others, the progress under
//! the one playing), the number and name in bold, a line, the length and the synopsis in grey. A card plays its episode.
use leptos::prelude::*;
use crate::services::tmdb::{fetch_details, fetch_season_details};

#[component]
pub fn EpisodesMenu(
    open: RwSignal<bool>,
    series_id: u32,
    current: (u32, u32),
    /// how far the episode playing is (0 to 1)
    #[prop(into)] progress: Signal<f64>,
    on_select_episode: Option<Callback<(u32, u32)>>,
) -> impl IntoView {
    let (current_season, current_episode) = current;
    let season = RwSignal::new(current_season);
    let picker_open = RwSignal::new(false);
    let seasons = LocalResource::new(move || async move { fetch_details(series_id, true).await.ok().and_then(|d| d.number_of_seasons).unwrap_or(1) });
    let episodes = LocalResource::new(move || {
        let n = season.get();
        async move { fetch_season_details(series_id, n).await.map(|s| s.episodes).unwrap_or_default() }
    });
    view! {
        <Show when=move || open.get()>
            <div class="absolute inset-0 z-50 bg-void text-ink" on:click=|e: leptos::ev::MouseEvent| e.stop_propagation()>
                <button type="button" on:click=move |_| open.set(false) class="absolute left-[calc(env(safe-area-inset-left)+8px)] top-0 flex h-14 w-14 items-center justify-center" aria-label=crate::i18n::t("player-back")>
                    <i class="ph ph-arrow-left text-heading"></i>
                </button>
                {move || (seasons.get().unwrap_or(1) > 1).then(|| view! {
                    <button type="button" on:click=move |_| picker_open.set(true) class="btn-pill absolute right-[calc(env(safe-area-inset-right)+8px)] top-2 h-[51px] min-w-[148px] justify-between gap-4 bg-sheet px-5 text-title-sm font-bold">
                        {move || crate::i18n::t_args("media-season-n", &[("n", fluent_bundle::FluentValue::from(season.get() as i64))])}
                        <i class="ph-fill ph-caret-down text-body"></i>
                    </button>
                })}
                <div class="absolute inset-x-0 bottom-0 top-[78px] flex snap-x gap-5 overflow-x-auto scrollbar-hide ps-[calc(env(safe-area-inset-left)+18px)] pe-6">
                    <Suspense fallback=|| view! { <span></span> }>
                        {move || episodes.get().map(|list| list.into_iter().map(|ep| {
                            let n = ep.episode_number;
                            let playing = move || season.get() == current_season && n == current_episode;
                            let still = ep.still_url("w500");
                            let pick = move |_| {
                                if !playing() {
                                    if let Some(cb) = on_select_episode { cb.run((season.get_untracked(), n)); }
                                }
                                open.set(false);
                            };
                            view! {
                                <button type="button" on:click=pick class="w-[234px] shrink-0 snap-start text-start cursor-pointer">
                                    <span class="relative block aspect-video w-full overflow-hidden rounded-tile bg-raised">
                                        {still.map(|s| view! { <img src=s alt="" loading="lazy" class="h-full w-full object-cover" /> })}
                                        <Show when=move || !playing()>
                                            <span class="absolute inset-0 flex items-center justify-center">
                                                <span class="flex h-[60px] w-[60px] items-center justify-center rounded-full border-2 border-ink bg-void/40"><i class="ph-fill ph-play text-heading"></i></span>
                                            </span>
                                        </Show>
                                        <Show when=playing>
                                            <span class="absolute inset-x-0 bottom-0 h-1 bg-ink/30"><span class="block h-full bg-brand" style=move || format!("width:{:.1}%", progress.get().clamp(0.0, 1.0) * 100.0)></span></span>
                                        </Show>
                                    </span>
                                    <span class="flex h-[54px] items-center border-b border-ink/20 text-title-sm font-bold"><span class="truncate">{format!("{n}. {}", ep.name)}</span></span>
                                    {ep.runtime.filter(|r| *r > 0).map(|r| view! { <span class="mt-2.5 block text-body-sm text-ink-body">{format!("{r}m")}</span> })}
                                    <span class="mt-1 line-clamp-5 block text-body-md leading-[19.4px] text-ink-body">{ep.overview.clone()}</span>
                                </button>
                            }
                        }).collect_view())}
                    </Suspense>
                </div>
                {move || view! { <crate::components::media::details::season_picker::SeasonPicker open=picker_open seasons=seasons.get().unwrap_or(1) season=season /> }}
            </div>
        </Show>
    }
}
