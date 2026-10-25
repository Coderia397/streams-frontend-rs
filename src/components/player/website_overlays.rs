//! What the website player draws over the picture besides its controls, as measured on Netflix's website player (note 126): the round
//! discs a skip or a play and pause leaves on the picture, and the pause screen ("You're watching") after a while paused and still.
use fluent_bundle::FluentValue;
use leptos::prelude::*;
use crate::components::media::icons::app_icons::{AppIcon, Glyph};

/// The disc an action leaves on the picture.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Disc {
    Back,
    Forward,
    Play,
    Pause,
}

/// The disc of the last skip or play and pause. Each action brings a new element (the number counts them), so a repeat starts the fade
/// again. Back sits 10.3% in from the left, forward 9.05% in from the right (centres, measured on a 1552 px wide player), play and pause
/// in the middle; all centred top to bottom.
#[component]
pub fn PlayerDiscs(disc: ReadSignal<Option<(Disc, u32)>>) -> impl IntoView {
    move || disc.get().map(|(kind, _)| {
        let (place, extra) = match kind {
            Disc::Back => ("left:calc(10.3% - 50px);top:50%;transform:translateY(-50%)", "player-disc-skip"),
            Disc::Forward => ("right:calc(9.05% - 50px);top:50%;transform:translateY(-50%)", "player-disc-skip"),
            Disc::Play | Disc::Pause => ("left:50%;top:50%;transform:translate(-50%,-50%)", ""),
        };
        let icon = match kind {
            Disc::Back => view! { <AppIcon glyph=Glyph::Back10 class="text-display-sm" /> }.into_any(),
            Disc::Forward => view! { <AppIcon glyph=Glyph::Forward10 class="text-display-sm" /> }.into_any(),
            Disc::Play => view! { <AppIcon glyph=Glyph::Play class="text-heading-xl" /> }.into_any(),
            Disc::Pause => view! { <i class="ph-fill ph-pause text-heading-xl"></i> }.into_any(),
        };
        view! {
            <div class=format!("player-disc {extra} absolute z-20 bg-player-disc") style=place aria-hidden="true">{icon}</div>
        }
    })
}

/// Netflix's pause screen: the picture dimmed to half, and on the left "You're watching", the title, the season, the episode and its
/// summary, with "Paused" at the bottom right. It comes and goes at once, with no fade (note 126). Placed as measured on a 1552 x 930
/// player: the text from 12% in and 38% down, "Paused" 10% from the right and the bottom.
#[component]
pub fn PauseScreen(
    shown: Signal<bool>,
    title: StoredValue<String>,
    season: Option<u32>,
    episode: Option<u32>,
    episode_name: Signal<Option<String>>,
    synopsis: Signal<Option<String>>,
) -> impl IntoView {
    let episode_line = move || {
        episode.map(|n| match episode_name.get() {
            Some(name) => crate::i18n::t_args("player-pause-episode", &[("name", FluentValue::from(name)), ("n", FluentValue::from(n as i64))]),
            None => crate::i18n::t_args("player-pause-episode-number", &[("n", FluentValue::from(n as i64))]),
        })
    };
    view! {
        <Show when=move || shown.get()>
            <div class="pointer-events-none absolute inset-0 z-[15] bg-player-dim" aria-live="polite">
                <div class="absolute left-[12%] top-[38%] flex max-w-[44%] flex-col items-start text-ink-soft">
                    <span class="text-title-sm text-ink-mist">{crate::i18n::t("player-youre-watching")}</span>
                    <span class="mt-1 text-display-lg leading-tight font-medium">{title.get_value()}</span>
                    {season.filter(|_| episode.is_some()).map(|s| view! {
                        <span class="mt-2 text-heading">{crate::i18n::t_args("media-season-n", &[("n", FluentValue::from(s as i64))])}</span>
                    })}
                    {move || episode_line().map(|line| view! { <span class="mt-6 text-heading font-bold">{line}</span> })}
                    {move || synopsis.get().filter(|s| !s.is_empty()).map(|s| view! {
                        <span class="mt-3 line-clamp-3 text-title-sm text-ink-silver">{s}</span>
                    })}
                </div>
                <span class="absolute bottom-[10%] right-[10%] text-title-sm text-ink-soft">{crate::i18n::t("player-paused")}</span>
            </div>
        </Show>
    }
}
