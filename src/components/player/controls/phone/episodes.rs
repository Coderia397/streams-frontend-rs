//! Episodes on the phone's player, as Netflix's Android app (its card layout and view tree, read on a Pixel 8, 2026-10-24, note 116):
//! the whole screen with the video paused under it, a back arrow at the top left and the season at the top right (it opens the season
//! menu); the season's episodes in a row that scrolls freely sideways (no snapping), clipped 8 px in from either side and from the
//! cutout, cards 254 px wide with 10 px round the content, so the stills are 234 px wide, 20 px apart, from 70 px in. A card: the still (16:9,
//! the play mark on the others, a 2 px bar of the part watched along the bottom of the one playing), the number and name on two lines at
//! most (bold for the one playing), a faint line, the length at half white and the synopsis in grey, which runs down to 10 px above the
//! bottom and ends there with an ellipsis. A card plays its episode.
use leptos::prelude::*;
use wasm_bindgen::JsCast;
use crate::components::media::icons::play_mark::PlayMark;
use crate::services::tmdb::{fetch_details, fetch_season_details};
use crate::components::media::icons::app_icons::{AppIcon, Glyph};

/// The synopsis's line, in px (45 device px on Netflix's).
const SYNOPSIS_LINE: f64 = 17.6;
/// The space under each card, in px.
const CARD_BOTTOM: f64 = 10.0;

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
    let list = NodeRef::<leptos::html::Div>::new();
    // how many synopsis lines fit above the card's bottom on this screen
    let lines = RwSignal::new(6.0_f64);
    let fit = move || {
        let Some(el) = list.get_untracked() else { return };
        let Ok(Some(first)) = el.query_selector("[data-synopsis]") else { return };
        let room = el.get_bounding_client_rect().bottom() - CARD_BOTTOM - first.get_bounding_client_rect().top();
        lines.set((room / SYNOPSIS_LINE).floor().max(1.0));
    };
    // the list opens on the episode playing, or on the first of another season (assumed from the app, note 116)
    let show_playing = move || {
        let Some(el) = list.get_untracked() else { return };
        let card = |sel: &str| el.query_selector(sel).ok().flatten().and_then(|c| c.dyn_into::<web_sys::HtmlElement>().ok());
        let left = match (card("[data-playing]"), card("[data-episode]")) {
            (Some(playing), Some(first)) => playing.offset_left() - first.offset_left(),
            _ => 0,
        };
        el.set_scroll_left(left);
    };
    // each time it opens, on the season playing
    Effect::new(move |was: Option<bool>| {
        let now = open.get();
        if now && was != Some(true) { season.set(current_season); }
        now
    });
    Effect::new(move |_| {
        if open.get() && episodes.get().is_some() {
            request_animation_frame(move || { fit(); show_playing(); });
        }
    });
    crate::utils::listen::listen_window(leptos::ev::resize, move |_| fit());

    view! {
        <Show when=move || open.get()>
            <div class="absolute inset-0 z-50 bg-scrim-menu text-ink" on:click=|e: leptos::ev::MouseEvent| e.stop_propagation()>
                // the arrow 16 px in from the edge the page may draw in, its button 52 by 56 px
                <button type="button" on:click=move |_| open.set(false) class="absolute left-[calc(env(safe-area-inset-left)+8px)] top-0 flex h-14 w-[52px] items-center ps-2" aria-label=crate::i18n::t("player-back")>
                    <AppIcon glyph=Glyph::Back class="text-heading" />
                </button>
                {move || (seasons.get().unwrap_or(1) > 1).then(|| view! {
                    <button type="button" on:click=move |_| picker_open.set(true) class="btn-pill absolute right-[calc(env(safe-area-inset-right)+16px)] top-2 h-[51px] min-w-[148px] justify-between gap-4 bg-menu-button ps-4 pe-[21px] text-title-sm font-bold text-ink-soft">
                        {move || crate::i18n::t_args("media-season-n", &[("n", fluent_bundle::FluentValue::from(season.get() as i64))])}
                        <i class="ph-fill ph-caret-down text-body"></i>
                    </button>
                })}
                // the row: its edges 8 px in, where the cards are cut off as they scroll
                <div
                    node_ref=list
                    class="absolute bottom-0 left-[calc(env(safe-area-inset-left)+8px)] right-[calc(env(safe-area-inset-right)+8px)] top-[68px] flex items-start overflow-x-auto overscroll-x-contain scrollbar-hide"
                    style=move || format!("--synopsis-lines: {}", lines.get())
                >
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
                                <button type="button" on:click=pick data-episode="" data-playing=move || playing().then_some("") class="flex w-[254px] shrink-0 flex-col p-2.5 text-start cursor-pointer">
                                    <span class="relative block aspect-video w-full overflow-hidden rounded-tile bg-raised">
                                        {still.map(|s| view! { <img src=s alt="" loading="lazy" decoding="async" class="h-full w-full object-cover" /> })}
                                        <Show when=move || !playing()>
                                            <span class="absolute inset-0 flex items-center justify-center"><PlayMark size=36.0 /></span>
                                        </Show>
                                        <Show when=playing>
                                            <span class="absolute inset-x-0 bottom-0 h-[2px] bg-episode-track"><span class="block h-full bg-brand" style=move || format!("width:{:.1}%", progress.get().clamp(0.0, 1.0) * 100.0)></span></span>
                                        </Show>
                                    </span>
                                    // the name: held to the height of two lines, 8 px under the still, and centred in them
                                    <span class="flex h-[53.8px] items-center pt-2 text-body-lg leading-[22.9px] text-ink-soft" class=("font-bold", playing)>
                                        <span class="line-clamp-2">{format!("{n}. {}", ep.name)}</span>
                                    </span>
                                    <span class="mt-2 block h-px bg-ink/15"></span>
                                    <span class="mt-2 block h-[14.4px] text-caption leading-[16px] text-ink/50">{ep.runtime.filter(|r| *r > 0).map(|r| format!("{r}m"))}</span>
                                    <span data-synopsis="" class="mt-2 line-clamp-(--synopsis-lines) text-body-sm leading-[17.6px] text-ink-body">{ep.overview.clone()}</span>
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
