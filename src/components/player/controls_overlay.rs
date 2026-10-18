use leptos::prelude::*;
use crate::components::player::timeline_scrubber::TimelineScrubber;

fn format_time_remaining(curr: f64, dur: f64) -> String {
    if dur <= 0.0 || curr.is_nan() || dur.is_nan() {
        return "00:00".to_string();
    }
    let rem = (dur - curr).max(0.0).floor() as u64;
    let hours = rem / 3600;
    let mins = (rem % 3600) / 60;
    let secs = rem % 60;
    if hours > 0 {
        format!("{}:{:02}:{:02}", hours, mins, secs)
    } else {
        format!("{:02}:{:02}", mins, secs)
    }
}

#[component]
pub fn ControlsOverlay(
    title: String,
    subtitle: Signal<String>,
    /// the next episode: number, name, summary and still, for the card that opens over the next episode button
    next_info: Signal<Option<(u32, String, String, Option<String>)>>,
    is_tv: bool,
    is_playing: ReadSignal<bool>,
    is_buffering: ReadSignal<bool>,
    is_muted: ReadSignal<bool>,
    volume: ReadSignal<f64>,
    current_time: ReadSignal<f64>,
    duration: ReadSignal<f64>,
    buffered_fraction: ReadSignal<f64>,
    is_idle: ReadSignal<bool>,
    on_toggle_play: Callback<()>,
    on_seek: Callback<f64>,
    on_skip_by: Callback<f64>,
    on_toggle_mute: Callback<()>,
    on_set_volume: Callback<f64>,
    on_toggle_episodes: Callback<()>,
    on_toggle_subtitles: Callback<()>,
    on_next_episode: Option<Callback<()>>,
    on_toggle_fullscreen: Callback<()>,
    on_back: Callback<()>,
) -> impl IntoView {
    // 36 px icons inside 44 px buttons, as in the app
    let play_icon_class = move || if is_playing.get() { "ph-fill ph-pause text-icon" } else { "ph-fill ph-play text-icon" };
    let speaker_icon_class = move || {
        if is_muted.get() || volume.get() <= 0.0 { "ph ph-speaker-x text-icon" } else if volume.get() > 0.5 { "ph ph-speaker-high text-icon" } else { "ph ph-speaker-low text-icon" }
    };
    let overlay_class = move || {
        format!(
            "absolute inset-0 z-20 transition-opacity duration-(--motion-slow) select-none {}",
            if is_idle.get() && is_playing.get() { "opacity-0 pointer-events-none cursor-none" } else { "opacity-100 pointer-events-auto cursor-default" }
        )
    };
    // every button: 44 x 44, no plate
    let btn = "flex h-11 relative w-11 shrink-0 items-center justify-center rounded-hairline text-ink transition-transform duration-(--motion-quick) hover:scale-125 active:scale-95";

    view! {
        <div class=overlay_class>
            // the top shade and the back and report buttons (30 px from the corner)
            <div class="pointer-events-none absolute inset-x-0 top-0 h-[85%]" style="background: linear-gradient(rgba(0,0,0,0.3) 0, rgba(0,0,0,0) 20%);" />
            <button on:click=move |_| on_back.run(()) class=format!("{} absolute left-[30px] top-[30px] z-10", btn.replace(" relative", "")) title=crate::i18n::t("controls-overlay-back-to-browse") aria-label=crate::i18n::t("controls-overlay-back-to-browse")>
                <i class="ph ph-arrow-left text-icon"></i>
            </button>
            <a href="/help/contact" class=format!("{} absolute right-[30px] top-[30px] z-10", btn.replace(" relative", "")) title=crate::i18n::t("controls-overlay-report-problem") aria-label=crate::i18n::t("controls-overlay-report-problem")>
                <i class="ph ph-flag text-icon-md"></i>
            </a>

            // the buffering ring in the middle
            <div class="pointer-events-none absolute inset-0 flex items-center justify-center">
                <Show when=move || is_buffering.get()>
                    <div class="h-16 w-16 animate-spin rounded-full border-4 border-brand border-t-transparent"></div>
                </Show>
            </div>

            // the bottom: a 135 px band with a soft shade, the timeline on top, the buttons 52 px below its top
            <div class="absolute inset-x-0 bottom-0 h-[135px]" style="background: linear-gradient(0deg, rgba(0,0,0,0.3) 0, rgba(0,0,0,0));">
                <div class="absolute left-[20px] right-[90px] top-0">
                    <TimelineScrubber current_time=current_time duration=duration buffered_fraction=buffered_fraction on_seek=on_seek />
                </div>
                <span class="absolute right-[28px] top-[1px] w-11 text-right text-body-md leading-5 text-ink">
                    {move || format_time_remaining(current_time.get(), duration.get())}
                </span>

                <div class="absolute left-[20px] right-[28px] top-[52px] flex h-11 items-center">
                    // left: play or pause, back 10, forward 10, volume
                    <div class="flex items-center gap-3 md:gap-[38px]">
                        <button on:click=move |_| on_toggle_play.run(()) class=btn title=move || if is_playing.get() { crate::i18n::t("player-pause") } else { crate::i18n::t("player-play") }>
                            <i class=play_icon_class></i>
                        </button>
                        <button on:click=move |_| on_skip_by.run(-10.0) class=btn title=crate::i18n::t("controls-overlay-rewind-10-seconds-left-arrow")>
                            <i class="ph ph-arrow-counter-clockwise text-icon"></i>
                            <span class="pointer-events-none absolute pt-[2px] text-caption font-black">"10"</span>
                        </button>
                        <button on:click=move |_| on_skip_by.run(10.0) class=btn title=crate::i18n::t("controls-overlay-forward-10-seconds-right-arrow")>
                            <i class="ph ph-arrow-clockwise text-icon"></i>
                            <span class="pointer-events-none absolute pt-[2px] text-caption font-black">"10"</span>
                        </button>
                        // the volume slider rises above the speaker while the pointer rests on it (a dark 34 x 185 box, accent bar, round knob)
                        <div class="group/vol relative flex items-center">
                            <button on:click=move |_| on_toggle_mute.run(()) class=btn title=move || if is_muted.get() { crate::i18n::t("player-unmute") } else { crate::i18n::t("player-mute") }>
                                <i class=speaker_icon_class></i>
                            </button>
                            <div class="absolute bottom-[44px] left-1/2 hidden h-[185px] w-[34px] -translate-x-1/2 items-center justify-center rounded-control bg-field group-hover/vol:flex group-focus-within/vol:flex">
                                <input
                                    type="range" min="0" max="1" step="0.02"
                                    prop:value=move || if is_muted.get() { 0.0 } else { volume.get() }
                                    on:input=move |e| { if let Ok(v) = event_target_value(&e).parse::<f64>() { on_set_volume.run(v); } }
                                    class="cursor-pointer accent-brand"
                                    style="writing-mode: vertical-lr; direction: rtl; height: 150px; width: 8px;"
                                    aria-label=crate::i18n::t("player-volume")
                                />
                            </div>
                        </div>
                    </div>

                    // the title, centred
                    <div class="mx-6 flex min-w-0 flex-1 items-baseline justify-center gap-3 text-title text-ink">
                        <span class="truncate font-bold">{title.clone()}</span>
                        {move || { let sub = subtitle.get(); (!sub.is_empty()).then(|| view! { <span class="truncate text-ink">{sub}</span> }) }}
                    </div>

                    // right: next, episodes, audio and subtitles, fullscreen
                    <div class="flex items-center gap-3 md:gap-[38px]">
                        <Show when=move || is_tv && on_next_episode.is_some()>
                            <div class="group/next relative">
                                <button on:click=move |_| { if let Some(cb) = on_next_episode { cb.run(()); } } class=btn title=crate::i18n::t("controls-overlay-next-episode")>
                                    <i class="ph ph-skip-forward text-icon"></i>
                                </button>
                                // the card: "Next episode", the still, number and name, four lines of the summary
                                {move || next_info.get().map(|(n, name, overview, still)| view! {
                                    <div class="pointer-events-none fixed bottom-[94px] right-0 z-30 hidden w-[min(700px,100vw)] flex-col bg-field shadow-2xl group-hover/next:flex">
                                        <span class="flex h-[73px] items-center px-[24px] text-heading-2xl font-medium text-ink bg-line">{crate::i18n::t("controls-overlay-next-episode")}</span>
                                        <span class="flex gap-[26px] px-[35px] py-[24px]">
                                            <span class="relative h-[140px] w-[248px] shrink-0 overflow-hidden bg-[#1a1a1a]">
                                                {still.map(|u| view! { <img src=u alt="" class="h-full w-full object-cover" /> })}
                                                <span class="absolute inset-0 flex items-center justify-center"><i class="ph-fill ph-play text-display text-ink"></i></span>
                                            </span>
                                            <span class="min-w-0 flex-1 text-ink">
                                                <span class="block truncate text-row font-medium">{format!("{}   {}", n, name)}</span>
                                                <span class="mt-1 block line-clamp-4 max-h-[100px] overflow-hidden text-title leading-[25px]">{overview}</span>
                                            </span>
                                        </span>
                                    </div>
                                })}
                            </div>
                        </Show>
                        <Show when=move || is_tv>
                            <button on:click=move |_| on_toggle_episodes.run(()) class=btn title=crate::i18n::t("controls-overlay-episode-selector")>
                                <i class="ph ph-stack text-icon"></i>
                            </button>
                        </Show>
                        <button on:click=move |_| on_toggle_subtitles.run(()) class=btn title=crate::i18n::t("controls-overlay-audio-subtitles")>
                            <i class="ph ph-subtitles text-icon"></i>
                        </button>
                        <button on:click=move |_| on_toggle_fullscreen.run(()) class=btn title=crate::i18n::t("controls-overlay-fullscreen-f")>
                            <i class="ph ph-corners-out text-icon"></i>
                        </button>
                    </div>
                </div>
            </div>
        </div>
    }
}
