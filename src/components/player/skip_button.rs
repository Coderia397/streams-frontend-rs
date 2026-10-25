use leptos::prelude::*;
use crate::services::ai_engine::SkipMarks;
use crate::components::media::icons::app_icons::{AppIcon, Glyph};

/// The buttons over the picture: Skip Recap, Skip Intro and, at the credits of an episode, "Watch credits" and "Next episode". When they show comes from the skip marks the engine guesses
/// (ratios of the video's length, so a stream with an extra logo or a different cut still lines up); a video without a mark has no button. They are the app's pills (btn-pill-md:
/// fully round, 40 px, grey glass; Next episode white). On the website they sit at the controls' right edge, above the controls band while it shows (--player-bar-h) and lower when
/// it hides, moving with it; on the phone they keep their corner.
#[component]
pub fn SkipButton(
    current_time: ReadSignal<f64>,
    duration: ReadSignal<f64>,
    is_tv: bool,
    marks: Signal<SkipMarks>,
    /// whether the website's controls show (the buttons keep above them)
    lifted: Signal<bool>,
    /// seek to this time (seconds)
    on_skip_to: Callback<f64>,
    on_next_episode: Callback<()>,
) -> impl IntoView {
    let inside = move |range: Option<[f64; 2]>| {
        let d = duration.get();
        let t = current_time.get();
        d > 0.0 && range.is_some_and(|r| t >= r[0] * d && t < r[1] * d - 1.0)
    };
    let show_recap = move || inside(marks.get().recap);
    let show_intro = move || inside(marks.get().intro);
    // from the credits mark to the last seconds of the episode
    let show_next_episode = move || {
        let d = duration.get();
        let t = current_time.get();
        is_tv && d > 0.0 && marks.get().credits.is_some_and(|c| t >= c * d && t < d - 2.0)
    };
    // "Watch credits" hides the end of episode offer for the rest of this play
    let (credits_dismissed, set_credits_dismissed) = signal(false);
    let base = "btn-pill btn-pill-md";
    let grey = "btn-glass";
    let form = crate::form::use_form();
    let place = move || {
        if form.get().is_phone() {
            "bottom-[3.54vw] right-[3vw]"
        } else if lifted.get() {
            "bottom-[calc(var(--player-bar-h)+1rem)] right-[28px]"
        } else {
            "bottom-[calc(var(--player-bar-h)/2)] right-[28px]"
        }
    };

    view! {
        <div class=move || format!("absolute {} z-30 flex items-center gap-3 pointer-events-auto transition-[bottom] duration-(--motion-slow)", place())>
            <Show when=show_recap>
                <button on:click=move |_| { if let Some(r) = marks.get().recap { on_skip_to.run(r[1] * duration.get()); } } class=format!("{base} {grey}")>
                    <i class="ph-fill ph-fast-forward"></i>
                    <span>{crate::i18n::t("player-skip-recap")}</span>
                </button>
            </Show>
            <Show when=show_intro>
                <button on:click=move |_| { if let Some(r) = marks.get().intro { on_skip_to.run(r[1] * duration.get()); } } class=format!("{base} {grey}")>
                    <i class="ph-fill ph-fast-forward"></i>
                    <span>{crate::i18n::t("player-skip-intro")}</span>
                </button>
            </Show>
            <Show when=move || show_next_episode() && !credits_dismissed.get()>
                <button on:click=move |_| set_credits_dismissed.set(true) class=format!("{base} {grey}")>
                    <span>{crate::i18n::t("player-watch-credits")}</span>
                </button>
                <button on:click=move |_| on_next_episode.run(()) class=format!("{base} btn-ink")>
                    <AppIcon glyph=Glyph::Play class="text-title" />
                    <span>{crate::i18n::t("player-next-episode")}</span>
                </button>
            </Show>
        </div>
    }
}
