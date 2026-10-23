use leptos::prelude::*;
use crate::services::ai_engine::SkipMarks;

/// The buttons over the picture: Skip Recap, Skip Intro and, at the credits of an episode, "Watch credits" and "Next episode". When they show comes from the skip marks the engine guesses
/// (ratios of the video's length, so a stream with an extra logo or a different cut still lines up); a video without a mark has no button. Laid out as read from the end of episode screen
/// on Netflix (1666 wide page): buttons 2.36vw high, 0.75vw apart, 3vw from the right edge and 3.54vw from the bottom, text 1vw, radius 4px.
#[component]
pub fn SkipButton(
    current_time: ReadSignal<f64>,
    duration: ReadSignal<f64>,
    is_tv: bool,
    marks: Signal<SkipMarks>,
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
    let base = "flex h-[2.36vw] items-center justify-center gap-[0.6vw] rounded-control px-[1.4vw] text-[1vw] font-medium select-none transition-colors duration-(--motion-fast)";
    let grey = "bg-button-secondary/70 text-ink hover:bg-button-secondary/40";

    view! {
        <div class="fixed bottom-[3.54vw] right-[3vw] z-30 flex items-center gap-[0.75vw] pointer-events-auto">
            <Show when=show_recap>
                <button on:click=move |_| { if let Some(r) = marks.get().recap { on_skip_to.run(r[1] * duration.get()); } } class=format!("{base} {grey}")>
                    <i class="ph-fill ph-fast-forward text-[1.4vw]"></i>
                    <span>{crate::i18n::t("player-skip-recap")}</span>
                </button>
            </Show>
            <Show when=show_intro>
                <button on:click=move |_| { if let Some(r) = marks.get().intro { on_skip_to.run(r[1] * duration.get()); } } class=format!("{base} {grey}")>
                    <i class="ph-fill ph-fast-forward text-[1.4vw]"></i>
                    <span>{crate::i18n::t("player-skip-intro")}</span>
                </button>
            </Show>
            <Show when=move || show_next_episode() && !credits_dismissed.get()>
                <button on:click=move |_| set_credits_dismissed.set(true) class=format!("{base} {grey}")>
                    <span>{crate::i18n::t("player-watch-credits")}</span>
                </button>
                <button on:click=move |_| on_next_episode.run(()) class=format!("{base} bg-ink text-void hover:bg-ink/75")>
                    <i class="ph-fill ph-play text-[1.6vw]"></i>
                    <span>{crate::i18n::t("player-next-episode")}</span>
                </button>
            </Show>
        </div>
    }
}
